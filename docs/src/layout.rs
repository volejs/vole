use maud::{DOCTYPE, Markup, PreEscaped, html};
use maudit::content::MarkdownHeading;
use maudit::route::PageContext;

use crate::content::SECTIONS;

pub fn badge(label: &str, color: &str) -> Markup {
    html! { span.badge data-color=(color) { (label) } }
}

pub fn render(
    ctx: &mut PageContext,
    title: &str,
    description: &str,
    content: Markup,
    headings: &[MarkdownHeading],
    section: Option<&str>,
) -> Markup {
    let style = ctx.assets.add_style_unchecked("src/styles.css");
    let header_style = ctx.assets.add_style_unchecked("src/header.css");
    let script = ctx.assets.add_script_unchecked("src/site.js");
    let current_path = ctx.current_path.trim_end_matches('/').to_owned();
    let is_home = current_path.is_empty();
    let is_examples_index = current_path == "/examples";
    let examples_style = is_examples_index.then(|| ctx.assets.add_style_unchecked("src/examples.css"));
    let examples_script = is_examples_index.then(|| ctx.assets.add_script_unchecked("src/examples.js"));
    let home_style = is_home.then(|| ctx.assets.add_style_unchecked("src/home.css"));
    let home_script = is_home.then(|| ctx.assets.add_script_unchecked("src/home.js"));
    let is_docs =
        section.is_some_and(|key| key == "docs" || SECTIONS.iter().any(|item| item.0 == key));
    html! {
        (DOCTYPE)
        html lang="zh-CN" data-docs[is_docs] {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="description" content=(description);
                (maudit::maud::generator())
                title { @if title == "Vole" { "Vole · ECMAScript runtime" } @else { (title) " · Vole" } }
                link rel="icon" href="/logo.svg" type="image/svg+xml";
                link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=DM+Sans:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500&display=swap";
                @if let Some(url) = ctx.canonical_url() { link rel="canonical" href=(url); }
                script { (PreEscaped("try{const t=localStorage.getItem('vole-theme');document.documentElement.dataset.theme=t||(matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light')}catch{}")) }
                (style)
                (header_style)
                (script)
                @if let Some(style) = home_style { (style) }
                @if let Some(script) = home_script { (script) }
                @if let Some(style) = examples_style { (style) }
                @if let Some(script) = examples_script { (script) }
            }
            body data-page=(if is_home { "home" } else { "content" }) {
                a.skip-link href="#main" { "跳转到正文" }
                (crate::header::render(is_home, is_docs, section))
                div class=(if is_home { "home-layout" } else if is_docs { "docs-layout" } else { "content-layout" }) {
                    aside.sidebar id="sidebar" aria-label="站点导航" {
                        nav {
                            section.nav-section.site-navigation {
                                h2 { "Vole" }
                                ul {
                                    li { a href="/" { "首页" } }
                                    li { a href=(SECTIONS[0].2) { "Docs" } }
                                    @if !is_docs {
                                        @for (_, name, href) in SECTIONS {
                                            li { a.docs-child href=(href) { (name) } }
                                        }
                                    }
                                    li { a href="/examples/" { "Examples" } }
                                    li { a href="/blog/" { "Blog" } }
                                    li { a href="/docs/#install" { "Install" } }
                                }
                            }
                            @if is_docs {
                                (crate::sidebar::render(ctx, if section == Some("docs") { "runtime" } else { section.unwrap_or("runtime") }, &current_path))
                            }
                        }
                        p.sidebar-footer { "Built with " a href="https://maudit.org/" { "Maudit" } }
                    }
                    main id="main" tabindex="-1" { article { (content) } }
                    @if is_docs {
                    aside.toc aria-label="本页目录" {
                        @if headings.iter().any(|heading| (2..=3).contains(&heading.level)) {
                            h2 { "本页目录" }
                            nav {
                                @for heading in headings.iter().filter(|heading| (2..=3).contains(&heading.level)) {
                                    a href=(format!("#{}", heading.id)) data-level=(heading.level) { (heading.title) }
                                }
                            }
                        }
                    }
                    }
                }
                @if !is_docs {
                    footer.site-footer {
                        span { "Vole" }
                        a href="https://github.com/volejs/vole" { "GitHub" }
                        a href="https://maudit.org/" { "Built with Maudit" }
                    }
                }
                dialog.search-dialog id="site-search" aria-labelledby="search-title" {
                    form.search-form method="dialog" {
                        label.visually-hidden id="search-title" for="search-input" { "搜索文档" }
                        input id="search-input" type="search" placeholder="搜索文档、模块或 API…" autocomplete="off";
                        button type="submit" aria-label="关闭搜索" { "Esc" }
                    }
                    p.search-status role="status" aria-live="polite" { "输入关键词搜索文档" }
                    ul.search-results aria-label="搜索结果" {}
                }
                dialog.image-preview aria-label="图片预览" { img alt=""; }
                div.toast role="status" aria-live="polite" {}
            }
        }
    }
}
