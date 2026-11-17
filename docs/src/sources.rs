use std::collections::HashSet;
use maudit::content::{Entry, glob_markdown_with_options, parse_markdown_with_frontmatter};

use crate::{content::{Document, document_path}, markdown, navigation};

pub fn load(section: &str) -> Vec<Entry<Document>> {
    let entries = glob_markdown_with_options::<Document>(
        &format!("content/{section}/*.md"), markdown::options(section),
    );
    let mut paths = HashSet::new();
    for entry in &entries {
        let path = document_path(section, &entry.id);
        assert!(paths.insert(path.clone()), "Duplicate page route: {path}");
        let data = parse_markdown_with_frontmatter::<Document>(entry.raw_content.as_deref().unwrap_or(""));
        assert!(!data.title.trim().is_empty(), "Missing title: {section}/{}", entry.id);
        if section == "examples" {
            assert!(data.category.is_some(), "Example requires category: {}", entry.id);
        }
        if navigation::section(section).is_some() {
            assert!(navigation::position(section, &path).is_some(), "Page is missing from navigation: {path}");
        }
    }
    if let Some(config) = navigation::section(section) {
        assert!(paths.contains(config.href), "Missing section entry page: {}", config.href);
        let mut links = HashSet::new();
        for group in config.groups {
            assert!(!group.links.is_empty(), "Empty navigation group: {}", group.title);
            for link in group.links {
                assert!(links.insert(link.href), "Duplicate navigation target: {}", link.href);
                assert!(paths.contains(link.href), "Missing navigation target: {}", link.href);
            }
        }
    }
    entries
}
