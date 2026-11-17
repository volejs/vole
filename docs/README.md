# Vole documentation

This site uses [Maudit](https://maudit.org/) with Rust routes, Maud templates,
Markdown content sources, and Syntect syntax highlighting.

## Development

Install Rust 1.97 or newer and the Maudit CLI:

```sh
cargo install maudit-cli --locked
cd docs
maudit dev
```

The development server watches Rust, content and assets and refreshes pages after
Maudit rebuilds them. Its local URL is printed in the terminal.

## Build

```sh
cd docs
cargo run --locked
```

Output is written to `docs/dist/`. Deploy this directory to any static host.
Set `SITE_URL=https://your-docs-host` to generate canonical URLs and a sitemap.
Use `maudit preview` to preview the production output.

The optional `bun run dev`, `bun run build`, and `bun run typecheck` commands are
aliases for the Rust/Maudit commands. No JavaScript package installation is needed.

## Structure

The homepage (`/`) has an independent layout. Top-level navigation contains:

- `Docs` (`/docs/`): Runtime, Package Manager, Modules, and Test.
- `Examples` (`/examples/`): example index and Markdown example pages.
- `Blog` (`/blog/`): article index and Markdown blog posts.

Only Docs pages display the documentation sidebar and table of contents.
Existing document URLs such as `/runtime/vm/` and `/modules/fs/` are preserved.
Examples and Blog use standalone layouts. Search includes all public collections.

- `src/main.rs`: Maudit entrypoint and content sources.
- `src/pages.rs`: home, documentation, Markdown source, search index and 404 routes.
- `src/indexes.rs`: Docs, Examples and Blog indexes.
- `src/content.rs`: frontmatter schema, example categories and stable slugs.
- `src/navigation.rs`: Docs section entrypoints, sidebar groups and pagination order.
- `src/sidebar.rs`: the active Docs section's sidebar and shared project links.
- `src/sources.rs`: content loading and navigation/category validation.
- `src/layout.rs`: shared navigation, table of contents and templates.
- `src/markdown.rs`: Markdown links, alerts and native Maudit shortcodes.
- `src/styles.css` and `src/site.js`: styling and progressive interactions.
- `src/examples.css` and `src/examples.js`: example catalog filters, loaded only on its index.
- `content/{runtime,pm,modules,test}/*.md`: existing Vole documentation.
- `content/examples/*.md`: examples, published at `/examples/{slug}/`.
- `content/blog/*.md`: posts, published at `/blog/{slug}/`.
- `public/`: unprocessed static assets.

Numeric filename prefixes are omitted from URLs:
`content/modules/05.fs.md` generates `/modules/fs/`.
Relative Markdown links such as `[File system](fs.md)` are rewritten to these URLs.

Docs sidebar labels and ordering come from `src/navigation.rs`, not filenames.
Pagination follows that same ordering within the active section. Add new Docs
pages to this configuration; existing document URLs do not need to change.
Shared contribution links point to the repository rather than duplicate pages.

## Examples and posts

Examples require a category as well as a title:

```md
---
title: Read a text file
description: Read UTF-8 text with vole:fs.
category: file-system
---

# Read a text file
```

Supported category IDs are `basics`, `modules-and-packages`, `network`,
`file-system`, `system`, `web-apis`, `built-in-modules`, `encoding`,
`cryptography` and `testing`. The catalog groups examples by category and offers
client-side keyword/category filters with shareable query parameters, such as
`/examples/?category=encoding`. Without JavaScript all populated groups remain
readable. Example pagination stays within the current category.

Posts use `title` and optional `description`, without a required category.
Indexes are populated automatically when Markdown files are added.
Blog entries are listed by filename in descending order; use date-prefixed names
such as `2026-11-17-runtime-notes.md` to keep them in chronological order.

## Content validation

Content loading rejects duplicate document routes, missing section entrypoints,
duplicate/missing navigation targets, unlisted Docs pages, empty titles and
missing/unknown example categories. Search entries include collection and
example-category metadata.

Document only implemented runtime capabilities. TypeScript execution is not type
checking, package downloads do not imply Node.js compatibility, and placeholder
APIs must state their limitations. See `/runtime/compatibility/`.

## Markdown

GFM tables, task lists, alerts, footnotes and highlighted code fences use Maudit's
built-in Markdown renderer. Component examples use native Maudit shortcodes:

```md
{{ badge label="Stable" color="success" / }}

{{ callout type="info" title="Note" }}
Content supports **Markdown** and `inline code`.
{{ /callout }}

{{ image src="/logo.svg" alt="Vole" zoom="true" / }}
```

