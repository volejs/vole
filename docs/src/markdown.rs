use maud::{PreEscaped, html};
use maudit::content::shortcodes::MarkdownShortcodes;
use maudit::content::{
    BlockQuoteKind, BlockquoteComponent, LinkComponent, LinkType, MarkdownComponents,
    MarkdownOptions, render_markdown,
};
use url::Url;

use crate::content::slug;
use crate::layout;

pub fn options(section: &str) -> MarkdownOptions {
    let mut shortcodes = MarkdownShortcodes::new();
    shortcodes.register("badge", |args, _| {
        layout::badge(
            args.get_str_or("label", ""),
            args.get_str_or("color", "primary"),
        )
        .into_string()
    });
    shortcodes.register("avatar", |args, _| {
        let name = args.get_str_or("name", "?");
        let body = html! {
            @if let Some(src) = args.get_str("src") { img src=(src) alt="" loading="lazy"; }
            @else { span aria-hidden="true" { (name.chars().next().unwrap_or('?')) } }
        };
        html! {
            @if let Some(href) = args.get_str("href") {
                a.avatar href=(href) aria-label=(name) { (body) }
            } @else { span.avatar aria-label=(name) { (body) } }
        }
        .into_string()
    });
    shortcodes.register("callout", |args, ctx| {
        let kind = args.get_str_or("type", "info");
        let title = args.get_str_or("title", kind);
        let body = render_markdown(args.get_str_or("body", ""), None, None, ctx);
        html! {
            div.callout data-kind=(kind) role=(if matches!(kind, "warning" | "error") { "alert" } else { "note" }) {
                strong { (title) }
                div { (PreEscaped(body)) }
            }
        }.into_string()
    });
    shortcodes.register("card", |args, ctx| {
        let body = render_markdown(args.get_str_or("body", ""), None, None, ctx);
        html! {
            div.content-card {
                @if let Some(eyebrow) = args.get_str("eyebrow") { p.eyebrow { (eyebrow) } }
                @if let Some(title) = args.get_str("title") {
                    h3 {
                        @if let Some(href) = args.get_str("href") { a href=(href) { (title) } }
                        @else { (title) }
                    }
                }
                (PreEscaped(body))
            }
        }
        .into_string()
    });
    shortcodes.register("image", |args, _| {
        html! {
            figure {
                img src=(args.get_str_required("src")) alt=(args.get_str_or("alt", "")) loading="lazy" data-zoom[args.get_or("zoom", false)];
                @if let Some(caption) = args.get_str("caption") { figcaption { (caption) } }
            }
        }.into_string()
    });
    shortcodes.register("video", |args, _| {
        html! {
            figure {
                video src=(args.get_str_required("src")) poster=[args.get_str("poster")] controls preload="metadata" {}
                @if let Some(title) = args.get_str("title") { figcaption { (title) } }
            }
        }.into_string()
    });
    MarkdownOptions {
        components: MarkdownComponents::new()
            .link(DocLink {
                section: section.into(),
            })
            .blockquote(Alert),
        shortcodes,
        ..Default::default()
    }
}

struct DocLink {
    section: String,
}

impl LinkComponent for DocLink {
    fn render_start(&self, href: &str, title: Option<&str>, _: LinkType) -> String {
        let href = self.rewrite(href);
        let attributes = html! { a href=(href) title=[title] {} }.into_string();
        attributes.trim_end_matches("</a>").to_string()
    }
}

impl DocLink {
    fn rewrite(&self, href: &str) -> String {
        if href.starts_with('#') || Url::parse(href).is_ok() || href.starts_with("//") {
            return href.into();
        }
        let base = Url::parse(&format!("https://docs.invalid/{}/index", self.section)).unwrap();
        let Ok(url) = base.join(href) else {
            return href.into();
        };
        if !url.path().ends_with(".md") {
            return href.into();
        }
        let path = url
            .path()
            .trim_end_matches(".md")
            .split('/')
            .map(slug)
            .collect::<Vec<_>>()
            .join("/");
        let query = url
            .query()
            .map(|value| format!("?{value}"))
            .unwrap_or_default();
        let fragment = url
            .fragment()
            .map(|value| format!("#{value}"))
            .unwrap_or_default();
        format!("{path}/{query}{fragment}")
    }
}

struct Alert;

impl BlockquoteComponent for Alert {
    fn render_start(&self, kind: Option<BlockQuoteKind>) -> String {
        match kind {
            Some(kind) => format!(
                "<blockquote class=\"callout\" data-kind=\"{}\"><strong>{}</strong>",
                kind.as_str(),
                kind.as_str()
            ),
            None => "<blockquote>".into(),
        }
    }
}
