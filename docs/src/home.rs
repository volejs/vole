use maud::{Markup, html};
use maudit::route::PageContext;

use crate::layout;

pub fn render(ctx: &mut PageContext, sections: &[(&str, &str, &str)]) -> Markup {
    layout::render(
        ctx,
        "Vole",
        "A small ECMAScript runtime built with QuickJS and Rust. Standard Web APIs, native system modules, and TypeScript execution.",
        html! {
            section.home-hero aria-labelledby="hero-title" {
                canvas.hero-atmosphere aria-hidden="true" {}
                div.hero-inner {
                    div.hero-copy {
                        a.hero-announcement href="/modules/globals/" {
                            span { "WinterTC" }
                            "Web standards, by default"
                            img src="/icons/arrow-right.svg" alt="" width="14" height="14";
                        }
                        h1 id="hero-title" { "Vole." br; "Small by design." }
                        p.hero-description {
                            "A small, open-source JavaScript runtime built with QuickJS and Rust. "
                            "Standard Web APIs, TypeScript execution, and native system modules."
                        }
                        p.hero-principle { "A small core. Explicit capabilities." }
                        div.hero-actions {
                            a.hero-button href="/docs/" {
                                img.hero-vole-icon src="/logo.svg" alt="" width="21" height="21";
                                "Get started"
                                img.hero-arrow src="/icons/arrow-right.svg" alt="" width="17" height="17";
                            }
                            a.hero-button href="https://github.com/volejs/vole" target="_blank" rel="noreferrer" {
                                img src="/icons/github.svg" alt="" width="21" height="21";
                                "GitHub"
                                img.hero-arrow src="/icons/arrow-right.svg" alt="" width="17" height="17";
                            }
                        }
                    }
                    div.hero-capabilities aria-label="Built-in Vole capabilities" {
                        div.capability-window {
                            div.capability-highlight aria-hidden="true" {}
                            ul.capability-track {
                                @for (index, (label, path)) in [
                                    ("TypeScript execution", "/runtime/vm/"),
                                    ("package manager", "/pm/overview/"),
                                    ("test runner", "/test/overview/"),
                                    ("native modules", "/modules/fs/"),
                                    ("standard Web APIs", "/modules/globals/"),
                                    ("async runtime", "/runtime/vm/"),
                                    ("streams & fetch", "/modules/globals/"),
                                    ("Web Crypto", "/modules/globals/"),
                                    ("extension system", "/runtime/extensions/"),
                                ].iter().enumerate() {
                                    li data-active[index == 4] {
                                        a href=(path) {
                                            span { (label) }
                                            img src="/icons/check.svg" alt="" width="17" height="17";
                                        }
                                    }
                                }
                            }
                        }
                        p.capability-caption { "Built in. Ready when you are." }
                    }
                }
                div.hero-foundations {
                    span { "QUICKJS ENGINE" }
                    span { "RUST FOUNDATION" }
                    span { "WINTERTC FIRST" }
                    span { "OPEN SOURCE" }
                }
            }
            section.home-code-section aria-labelledby="home-code-title" {
                div.home-section-heading {
                    p.eyebrow { "JAVASCRIPT, WITHOUT THE EXTRA WEIGHT" }
                    h2 id="home-code-title" { "Familiar APIs." br; "Native capabilities." }
                    p { "Use the Web APIs you already know. Import system capabilities explicitly, through dedicated vole:* modules." }
                    a.home-text-link href="/modules/globals/" { "Explore the APIs" span aria-hidden="true" { " →" } }
                }
                div.home-code-example {
                    div.home-code-heading { span { "main.ts" } span { "JavaScript + TypeScript" } }
                    pre { code {
                        "import { readTextFile } from \"vole:fs\";\n\n"
                        "const text = await readTextFile(\"package.json\");\n"
                        "const response = new Response(text, {\n"
                        "  headers: { \"content-type\": \"application/json\" },\n"
                        "});\n\n"
                        "console.log(await response.json());"
                    } }
                    div.home-run-command { span aria-hidden="true" { "$" } code { "vole run main.ts" } }
                }
            }
            section.home-explore aria-labelledby="home-explore-title" {
                div.home-section-heading {
                    p.eyebrow { "GET TO KNOW VOLE" }
                    h2 id="home-explore-title" { "One runtime." br; "Clear building blocks." }
                }
                div.home-doc-links {
                    @for (index, (key, title, path)) in sections.iter().enumerate() {
                        a.home-doc-link href=(path) {
                            span.home-doc-number { (format!("{:02}", index + 1)) }
                            div {
                                h3 { (title) }
                                p {
                                    @match *key {
                                        "runtime" => { "Explore the VM, module loading, and extension system." }
                                        "pm" => { "Initialize a project and install packages from npm." }
                                        "modules" => { "Work with standard Web APIs and native system modules." }
                                        _ => { "Write tests with the built-in runner and async lifecycle." }
                                    }
                                }
                            }
                            span.home-doc-arrow aria-hidden="true" { "↗" }
                        }
                    }
                }
            }
        },
        &[],
        None,
    )
}
