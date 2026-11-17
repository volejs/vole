use maud::{Markup, html};

use crate::content::SECTIONS;

pub fn render(is_home: bool, is_docs: bool, section: Option<&str>) -> Markup {
    html! {
        @if is_home {
            a.site-announcement href="/docs/" {
                span { "Built with QuickJS and Rust. Designed around standard Web APIs." }
                img src="/icons/arrow-right.svg" alt="" width="15" height="15";
            }
        }
        header.site-header data-docs[is_docs] {
            div.header-main {
                a.header-brand href="/" aria-label="Vole 首页" {
                    img src="/logo.svg" alt="" width="34" height="34";
                    span { "Vole" }
                }
                nav.header-nav aria-label="主导航" {
                    a href=(SECTIONS[0].2) aria-current=[if is_docs { Some("page") } else { None }] { "Docs" }
                    a href="/examples/" aria-current=[if section == Some("examples") { Some("page") } else { None }] { "Examples" }
                    a href="/blog/" aria-current=[if section == Some("blog") { Some("page") } else { None }] { "Blog" }
                }
                button.header-search type="button" data-search-open aria-label="搜索文档、API 和示例" aria-haspopup="dialog" aria-controls="site-search" title="搜索 (/)" {
                    img src="/icons/search.svg" alt="" width="17" height="17";
                    span { "Search docs, APIs, examples…" }
                    kbd { "/" }
                }
                div.header-utilities {
                    a.header-icon.header-github href="https://github.com/volejs/vole" target="_blank" rel="noreferrer" aria-label="GitHub" title="GitHub" {
                        img src="/icons/github.svg" alt="" width="18" height="18";
                    }
                    button.header-icon type="button" data-theme-toggle aria-label="切换主题" title="切换主题 (D)" {
                        img src="/icons/sun-moon.svg" alt="" width="19" height="19";
                    }
                    a.header-install href="/docs/#install" { "Install" }
                    button.header-icon.header-menu-toggle type="button" data-menu-toggle aria-controls="sidebar" aria-expanded="false" aria-label="展开导航" {
                        img.menu-open src="/icons/menu.svg" alt="" width="21" height="21";
                        img.menu-close src="/icons/x.svg" alt="" width="21" height="21";
                    }
                }
            }
            @if is_docs {
                nav.header-subnav aria-label="文档分类" {
                    div.header-subnav-inner {
                        @for (key, name, href) in SECTIONS {
                            @let icon = match key {
                                "runtime" => "cpu",
                                "pm" => "package",
                                "modules" => "boxes",
                                _ => "flask-conical",
                            };
                            a href=(href) aria-current=[if section == Some(key) { Some("page") } else { None }] {
                                img src=(format!("/icons/{icon}.svg")) alt="" width="14" height="14";
                                span { (name) }
                            }
                        }
                    }
                }
            }
        }
    }
}
