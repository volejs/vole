use maudit::content::{Entry, markdown_entry};
use maudit::route::PageContext;
use serde::{Deserialize, Serialize};
use crate::navigation::DOC_SECTIONS;

pub const SECTIONS: [(&str, &str, &str); 4] = [
    (DOC_SECTIONS[0].key, DOC_SECTIONS[0].title, DOC_SECTIONS[0].href),
    (DOC_SECTIONS[1].key, DOC_SECTIONS[1].title, DOC_SECTIONS[1].href),
    (DOC_SECTIONS[2].key, DOC_SECTIONS[2].title, DOC_SECTIONS[2].href),
    (DOC_SECTIONS[3].key, DOC_SECTIONS[3].title, DOC_SECTIONS[3].href),
];

pub const COLLECTIONS: [(&str, &str, &str); 6] = [
    SECTIONS[0],
    SECTIONS[1],
    SECTIONS[2],
    SECTIONS[3],
    ("examples", "Examples", "/examples/"),
    ("blog", "Blog", "/blog/"),
];

#[markdown_entry]
pub struct Document {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub sidebar: Sidebar,
    #[serde(default)]
    pub badges: Vec<Badge>,
    #[serde(default)]
    pub category: Option<ExampleCategory>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExampleCategory {
    Basics, ModulesAndPackages, Network, FileSystem, System,
    WebApis, BuiltInModules, Encoding, Cryptography, Testing,
}

impl ExampleCategory {
    pub const ALL: [Self; 10] = [Self::Basics, Self::ModulesAndPackages, Self::Network,
        Self::FileSystem, Self::System, Self::WebApis, Self::BuiltInModules,
        Self::Encoding, Self::Cryptography, Self::Testing];

    pub fn id(self) -> &'static str {
        match self {
            Self::Basics => "basics", Self::ModulesAndPackages => "modules-and-packages",
            Self::Network => "network", Self::FileSystem => "file-system", Self::System => "system",
            Self::WebApis => "web-apis", Self::BuiltInModules => "built-in-modules",
            Self::Encoding => "encoding", Self::Cryptography => "cryptography", Self::Testing => "testing",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Basics => "Basics", Self::ModulesAndPackages => "Modules and package management",
            Self::Network => "Network", Self::FileSystem => "File system", Self::System => "System",
            Self::WebApis => "Web standard APIs", Self::BuiltInModules => "Built-in modules",
            Self::Encoding => "Encoding", Self::Cryptography => "Cryptography", Self::Testing => "Testing",
        }
    }
}

#[derive(Default, Deserialize)]
pub struct Sidebar {
    pub label: Option<String>,
    pub badge: Option<Badge>,
}

#[derive(Deserialize)]
pub struct Badge {
    pub label: String,
    #[serde(default)]
    pub color: String,
}

pub fn slug(id: &str) -> &str {
    match id.split_once('.') {
        Some((prefix, rest))
            if !prefix.is_empty() && prefix.bytes().all(|b| b.is_ascii_digit()) =>
        {
            rest
        }
        _ => id,
    }
}

pub fn document_path(section: &str, id: &str) -> String {
    format!("/{section}/{}/", slug(id))
}

pub fn entries(ctx: &PageContext, section: &str) -> Vec<Entry<Document>> {
    let mut entries: Vec<_> = ctx
        .content::<Document>(section)
        .entries()
        .cloned()
        .collect();
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    if section == "blog" {
        entries.reverse();
    }
    entries
}

pub fn section_title(section: &str) -> &str {
    COLLECTIONS
        .iter()
        .find(|item| item.0 == section)
        .map(|item| item.1)
        .unwrap_or(section)
}
