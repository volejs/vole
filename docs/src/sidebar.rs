use maud::{Markup, html};
use maudit::route::PageContext;
use crate::{content::{document_path, entries}, layout, navigation};

pub fn render(ctx: &mut PageContext, key: &str, current_path: &str) -> Markup {
    let Some(section) = navigation::section(key) else { return html! {} };
    let documents = entries(ctx, key);
    html! {
        @for group in section.groups {
            section.nav-section {
                h2 { (group.title) }
                ul {
                    @for link in group.links {
                        li {
                            a href=(link.href) aria-current=[if link.href.trim_end_matches('/') == current_path { Some("page") } else { None }] {
                                span { (link.title) }
                                @if let Some(entry) = documents.iter().find(|entry| document_path(key, &entry.id) == link.href) {
                                    @if let Some(badge) = &entry.data(ctx).sidebar.badge { (layout::badge(&badge.label, &badge.color)) }
                                }
                            }
                        }
                    }
                }
            }
        }
        section.nav-section.project-links {
            h2 { (navigation::PROJECT_LINKS.title) }
            ul {
                @for link in navigation::PROJECT_LINKS.links {
                    li { a href=(link.href) target="_blank" rel="noreferrer" { span { (link.title) } span aria-hidden="true" { "↗" } } }
                }
            }
        }
    }
}
