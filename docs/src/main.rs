mod content;
mod header;
mod home;
mod indexes;
mod layout;
mod markdown;
mod navigation;
mod pages;
mod sidebar;
mod sources;

use indexes::{BlogIndex, DocsIndex, ExamplesIndex};
use maudit::{BuildOptions, BuildOutput, content_sources, coronate, routes};
use pages::{ContentPage, Home, MarkdownSource, NotFound, SearchIndex};

fn main() -> Result<BuildOutput, Box<dyn std::error::Error>> {
    std::env::set_current_dir(env!("CARGO_MANIFEST_DIR"))?;
    coronate(
        routes![
            Home,
            DocsIndex,
            ExamplesIndex,
            BlogIndex,
            ContentPage,
            MarkdownSource,
            SearchIndex,
            NotFound
        ],
        content_sources![
            "runtime" => sources::load("runtime"),
            "pm" => sources::load("pm"),
            "modules" => sources::load("modules"),
            "test" => sources::load("test"),
            "examples" => sources::load("examples"),
            "blog" => sources::load("blog")
        ],
        BuildOptions {
            static_dir: "public".into(),
            base_url: std::env::var("SITE_URL").ok(),
            ..Default::default()
        },
    )
}
