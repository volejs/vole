use maud::{PreEscaped, html};
use maudit::content::MarkdownContent;
use maudit::route::prelude::*;
use pulldown_cmark::{Event, Options, Parser};
use serde::Serialize;

use crate::content::{
    COLLECTIONS, Document, ExampleCategory, SECTIONS, document_path, entries, section_title, slug,
};
use crate::layout;

#[route("/")]
pub struct Home;

impl Route for Home {
    fn render(&self, ctx: &mut PageContext) -> impl Into<RenderResult> {
        crate::home::render(ctx, &SECTIONS)
    }
}

#[route("/[section]/[slug]")]
pub struct ContentPage;

#[derive(Params, Clone)]
pub struct ContentParams {
    pub section: String,
    pub slug: String,
}

impl Route<ContentParams> for ContentPage {
    fn pages(&self, ctx: &mut DynamicRouteContext) -> Pages<ContentParams> {
        COLLECTIONS
            .iter()
            .flat_map(|(section, _, _)| {
                ctx.content::<Document>(section).into_pages(|entry| {
                    Page::from_params(ContentParams {
                        section: (*section).into(),
                        slug: slug(&entry.id).into(),
                    })
                })
            })
            .collect()
    }

    fn render(&self, ctx: &mut PageContext) -> impl Into<RenderResult> {
        let params = ctx.params::<ContentParams>();
        let documents = entries(ctx, &params.section);
        let entry = documents
            .iter()
            .find(|entry| slug(&entry.id) == params.slug)
            .expect("Unknown document");
        let category = entry.data(ctx).category;
        let siblings = if crate::navigation::section(&params.section).is_some() {
            crate::navigation::links(&params.section)
                .filter_map(|link| documents.iter().find(|entry| document_path(&params.section, &entry.id) == link.href))
                .cloned().collect::<Vec<_>>()
        } else if params.section == "examples" {
            documents.iter().filter(|entry| entry.data(ctx).category == category).cloned().collect()
        } else { documents.clone() };
        let position = siblings.iter().position(|item| item.id == entry.id);
        let previous = position.and_then(|index| index.checked_sub(1)).and_then(|index| siblings.get(index));
        let next = position.and_then(|index| siblings.get(index + 1));
        let rendered = entry.render(ctx);
        let data = entry.data(ctx);
        let headings = data.get_headings().clone();
        let title = data.title.clone();
        let description = data.description.clone();
        let source = format!(
            "https://github.com/volejs/vole/edit/main/docs/content/{}/{}.md",
            params.section, entry.id
        );
        let body = html! {
            div.article-meta {
                span.eyebrow {
                    @if params.section == "examples" { a href="/examples/" { "Examples" } }
                    @else { (section_title(&params.section)) }
                    @if let Some(category) = category {
                        " / " a href=(format!("/examples/?category={}", category.id())) { (category.title()) }
                    }
                }
                button.copy-markdown type="button" data-copy-document=(format!("/content/{}/{}.md", params.section, entry.id)) {
                    "复制 Markdown"
                }
            }
            @if !data.badges.is_empty() {
                div.badges {
                    @for badge in &data.badges { (layout::badge(&badge.label, &badge.color)) }
                }
            }
            div.prose { (PreEscaped(rendered)) }
            footer.article-footer {
                a href=(source) target="_blank" rel="noreferrer" { "在 GitHub 上编辑此页" }
                nav.pagination aria-label="内容分页" {
                    @if let Some(previous) = previous {
                        a href=(document_path(&params.section, &previous.id)) {
                            small { "上一页" }
                            span { (previous.data(ctx).title) }
                        }
                    } @else { span {} }
                    @if let Some(next) = next {
                        a href=(document_path(&params.section, &next.id)) {
                            small { "下一页" }
                            span { (next.data(ctx).title) }
                        }
                    }
                }
            }
        };
        layout::render(
            ctx,
            &title,
            &description,
            body,
            &headings,
            Some(&params.section),
        )
    }
}

#[route("/search-index.json", sitemap(exclude = true))]
pub struct SearchIndex;

#[derive(Serialize)]
struct SearchDocument {
    title: String,
    section: String,
    path: String,
    text: String,
    collection: String,
    category: Option<ExampleCategory>,
    category_label: Option<&'static str>,
}

impl Route for SearchIndex {
    fn render(&self, ctx: &mut PageContext) -> impl Into<RenderResult> {
        let mut documents = Vec::new();
        for (section, name, _) in COLLECTIONS {
            for entry in entries(ctx, section) {
                let data = entry.data(ctx);
                let raw = entry.raw_content.as_deref().unwrap_or("");
                let text = Parser::new_ext(raw, Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
                    .filter_map(|event| match event {
                        Event::Text(text) | Event::Code(text) => Some(text.into_string()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                documents.push(SearchDocument {
                    title: data.title.clone(),
                    section: name.into(),
                    path: document_path(section, &entry.id),
                    text,
                    collection: section.into(),
                    category: data.category,
                    category_label: data.category.map(ExampleCategory::title),
                });
            }
        }
        serde_json::to_string(&documents).expect("Unable to serialize search index")
    }
}

#[route("/content/[section]/[file].md", sitemap(exclude = true))]
pub struct MarkdownSource;

#[derive(Params, Clone)]
pub struct SourceParams {
    pub section: String,
    pub file: String,
}

impl Route<SourceParams> for MarkdownSource {
    fn pages(&self, ctx: &mut DynamicRouteContext) -> Pages<SourceParams> {
        COLLECTIONS
            .iter()
            .flat_map(|(section, _, _)| {
                ctx.content::<Document>(section).into_pages(|entry| {
                    Page::from_params(SourceParams {
                        section: (*section).into(),
                        file: entry.id.clone(),
                    })
                })
            })
            .collect()
    }

    fn render(&self, ctx: &mut PageContext) -> impl Into<RenderResult> {
        let params = ctx.params::<SourceParams>();
        ctx.content::<Document>(&params.section)
            .get_entry(&params.file)
            .raw_content
            .clone()
            .unwrap_or_default()
    }
}

#[route("/404.html", sitemap(exclude = true))]
pub struct NotFound;

impl Route for NotFound {
    fn render(&self, ctx: &mut PageContext) -> impl Into<RenderResult> {
        layout::render(
            ctx,
            "页面不存在",
            "找不到请求的页面",
            html! {
                p.eyebrow { "404" }
                h1 { "页面不存在" }
                p { "可以从侧栏选择文档，或搜索需要的 API。" }
                a.button href="/" { "返回首页" }
            },
            &[],
            None,
        )
    }
}
