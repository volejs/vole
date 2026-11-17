use maud::{Markup, html};
use maudit::route::prelude::*;

use crate::content::{ExampleCategory, SECTIONS, document_path, entries, section_title};
use crate::layout;

#[route("/docs")]
pub struct DocsIndex;

impl Route for DocsIndex {
    fn render(&self, ctx: &mut PageContext) -> impl Into<RenderResult> {
        layout::render(
            ctx,
            "Docs",
            "Vole runtime、包管理器、标准模块和测试文档。",
            html! {
                p.eyebrow { "Documentation" }
                h1 { "Docs" }
                p { "了解 Vole 的运行时、系统能力和开发工具。" }
                div.collection-list {
                    @for (key, title, href) in SECTIONS {
                        a.collection-entry href=(href) {
                            h2 { (title) }
                            p {
                                @match key {
                                    "runtime" => { "VM 生命周期、模块加载与扩展系统。" }
                                    "pm" => { "项目初始化与依赖安装。" }
                                    "modules" => { "标准 Web API 和 vole:* 系统模块。" }
                                    _ => { "测试运行器、注册与异步生命周期。" }
                                }
                            }
                        }
                    }
                }
                section.install-section id="install" {
                    h2 { "Install Vole" }
                    p { "从源码构建 Vole，需要先安装 Rust 工具链。" }
                    pre { code { "git clone https://github.com/volejs/vole.git\ncd vole\ncargo build -p vole-cli\n./target/debug/vole --version" } }
                }
            },
            &[],
            Some("docs"),
        )
    }
}

#[route("/examples")]
pub struct ExamplesIndex;

impl Route for ExamplesIndex {
    fn render(&self, ctx: &mut PageContext) -> impl Into<RenderResult> {
        let documents = entries(ctx, "examples");
        let groups: Vec<_> = ExampleCategory::ALL.into_iter().map(|category| {
            let items: Vec<_> = documents.iter().filter_map(|entry| {
                let data = entry.data(ctx);
                if data.category == Some(category) {
                    Some((data.title.clone(), data.description.clone(), document_path("examples", &entry.id)))
                } else { None }
            }).collect();
            (category, items)
        }).collect();
        let content = html! {
            div.examples-page {
                header.collection-header {
                    h1 { "Examples" }
                    p { "从可阅读的小例子开始，了解 Vole 的运行时与系统能力。" }
                }
                form.examples-controls role="search" hidden {
                    label for="example-search" { "搜索示例" }
                    input id="example-search" type="search" placeholder="Search examples…" autocomplete="off";
                    div.example-filters role="group" aria-label="示例分类" {
                        button type="button" data-example-category="" aria-pressed="true" { "All" }
                        @for (category, items) in &groups {
                            button type="button" data-example-category=(category.id()) aria-pressed="false" disabled[items.is_empty()] {
                                (category.title()) span { (items.len()) }
                            }
                        }
                    }
                }
                p.examples-status role="status" aria-live="polite" { (documents.len()) " examples" }
                p.examples-empty hidden { "没有匹配的示例。" }
                @for (category, items) in groups {
                    @if !items.is_empty() {
                        section.examples-group id=(format!("examples-{}", category.id())) data-example-group=(category.id()) {
                            h2 { a href=(format!("#examples-{}", category.id())) { (category.title()) } }
                            div.example-grid {
                                @for (title, description, path) in items {
                                    a.example-entry href=(path) data-category=(category.id()) data-search=(format!("{title} {description} {}", category.title())) {
                                        h3 { (title) }
                                        p { (description) }
                                        span.example-link-label { "Read example →" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
        layout::render(ctx, "Examples", "按分类浏览 Vole 使用示例。", content, &[], Some("examples"))
    }
}

#[route("/blog")]
pub struct BlogIndex;

impl Route for BlogIndex {
    fn render(&self, ctx: &mut PageContext) -> impl Into<RenderResult> {
        collection_index(
            ctx,
            "blog",
            "Vole 的开发进展、技术文章与发布记录。",
            "暂无文章。",
        )
    }
}

fn collection_index(
    ctx: &mut PageContext,
    section: &str,
    description: &str,
    empty: &str,
) -> Markup {
    let items = entries(ctx, section);
    let title = section_title(section);
    let content = html! {
        header.collection-header {
            h1 { (title) }
            p { (description) }
        }
        div.collection-list {
            @if items.is_empty() {
                p.empty-state { (empty) }
            } @else {
                @for entry in items {
                    @let data = entry.data(ctx);
                    a.collection-entry href=(document_path(section, &entry.id)) {
                        h2 { (data.title) }
                        @if !data.description.is_empty() { p { (data.description) } }
                    }
                }
            }
        }
    };
    layout::render(ctx, title, description, content, &[], Some(section))
}
