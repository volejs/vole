pub struct Link {
    pub title: &'static str,
    pub href: &'static str,
}

pub struct Group {
    pub title: &'static str,
    pub links: &'static [Link],
}

pub struct Section {
    pub key: &'static str,
    pub title: &'static str,
    pub href: &'static str,
    pub groups: &'static [Group],
}

const fn link(title: &'static str, href: &'static str) -> Link {
    Link { title, href }
}

pub const DOC_SECTIONS: [Section; 4] = [
    Section {
        key: "runtime", title: "Runtime", href: "/runtime/welcome/",
        groups: &[
            Group { title: "Get Started", links: &[
                link("Welcome to Vole", "/runtime/welcome/"),
                link("Installation", "/runtime/installation/"),
                link("Quickstart", "/runtime/quickstart/"),
                link("TypeScript", "/runtime/typescript/"),
                link("Project setup", "/runtime/init/"),
            ] },
            Group { title: "Core Runtime", links: &[
                link("Vole Runtime", "/runtime/vm/"),
                link("Watch Mode", "/runtime/watch/"),
                link("REPL", "/runtime/repl/"),
                link("Troubleshooting", "/runtime/troubleshooting/"),
            ] },
            Group { title: "Module System", links: &[
                link("File Types", "/runtime/file-types/"),
                link("Module Resolution", "/runtime/module-resolution/"),
                link("Explicit Imports & Import Maps", "/runtime/import-maps/"),
                link("Configuration Modules", "/runtime/configuration-modules/"),
            ] },
            Group { title: "Async & Lifecycle", links: &[
                link("Async lifecycle", "/runtime/lifecycle/"),
            ] },
            Group { title: "Networking", links: &[
                link("Network overview", "/runtime/networking/"),
            ] },
            Group { title: "Interop", links: &[
                link("Environment Variables", "/runtime/environment/"),
                link("FFI", "/runtime/ffi/"),
            ] },
            Group { title: "Standards", links: &[
                link("API Overview", "/runtime/api-overview/"),
                link("Compatibility & Limits", "/runtime/compatibility/"),
            ] },
            Group { title: "Internals", links: &[
                link("Native module architecture", "/runtime/modules/"),
                link("Runtime extensions", "/runtime/extensions/"),
            ] },
        ],
    },
    Section {
        key: "pm", title: "Package Manager", href: "/pm/overview/",
        groups: &[
            Group { title: "Get Started", links: &[link("Overview", "/pm/overview/")] },
            Group { title: "Commands", links: &[
                link("vole init", "/pm/init/"), link("vole install", "/pm/install/"),
            ] },
            Group { title: "Configuration", links: &[link("Configuration & Limits", "/pm/configuration/")] },
        ],
    },
    Section {
        key: "modules", title: "Modules", href: "/modules/overview/",
        groups: &[
            Group { title: "Overview", links: &[link("Module overview", "/modules/overview/")] },
            Group { title: "Built-in Modules", links: &[
                link("vole:os", "/modules/os/"), link("vole:env", "/modules/env/"),
                link("vole:fs", "/modules/fs/"), link("vole:path", "/modules/path/"),
                link("vole:exec", "/modules/exec/"), link("vole:http", "/modules/http/"),
                link("vole:tcp", "/modules/tcp/"), link("vole:udp", "/modules/udp/"),
                link("vole:dns", "/modules/dns/"), link("vole:socket (Unix)", "/modules/socket/"),
                link("vole:signal", "/modules/signal/"), link("vole:ffi", "/modules/ffi/"),
                link("vole:until", "/modules/until/"), link("vole:test", "/modules/test/"),
                link("vole:wasm (partial)", "/modules/wasm/"), link("vole:tls (unavailable)", "/modules/tls/"),
            ] },
            Group { title: "Web APIs", links: &[link("Globals & Web APIs", "/modules/globals/")] },
        ],
    },
    Section {
        key: "test", title: "Test", href: "/test/overview/",
        groups: &[
            Group { title: "Get Started", links: &[link("Test runner", "/test/overview/")] },
            Group { title: "Execution", links: &[link("Test execution", "/test/execution/")] },
            Group { title: "Features", links: &[
                link("Registration & Hooks", "/test/api/"), link("Async lifecycle", "/test/lifecycle/"),
            ] },
            Group { title: "Output", links: &[link("Output & Exit Codes", "/test/output/")] },
        ],
    },
];

pub const PROJECT_LINKS: Group = Group { title: "Contributing", links: &[
    link("Roadmap", "https://github.com/volejs/vole/blob/main/ROADMAP.md"),
    link("Benchmarking", "https://github.com/volejs/vole/blob/main/benchmarks/README.md"),
    link("Contributor workflow", "https://github.com/volejs/vole/blob/main/AGENTS.md"),
    link("License", "https://github.com/volejs/vole/blob/main/LICENSE"),
] };

pub fn section(key: &str) -> Option<&'static Section> {
    DOC_SECTIONS.iter().find(|section| section.key == key)
}

pub fn links(key: &str) -> impl Iterator<Item = &'static Link> {
    section(key).into_iter().flat_map(|section| section.groups).flat_map(|group| group.links)
}

pub fn position(key: &str, href: &str) -> Option<usize> {
    links(key).position(|link| link.href == href)
}

