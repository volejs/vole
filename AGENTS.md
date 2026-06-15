# AGENTS.md

本文件面向开发与维护 vole runtime 的 Agent，汇总工作区结构、构建方式、架构约定和当前实现状态。

> **状态来源**：以 `ROADMAP.md` 为准；`MinimumCommonAPI.md` 已删除，不再维护。

---

## Workspace Structure

Cargo workspace with `resolver = "2"`, members under `crates/`:

| Crate                           | Package Name         | Type          | Purpose                                   |
| ------------------------------- | -------------------- | ------------- | ----------------------------------------- |
| `crates/core`                   | `vole-core`          | Library       | Shared JS binding support and runtime primitives |
| `crates/runtime`                | `vole-runtime`       | Library       | VM (async), extension registry, lifecycle |
| `crates/modules`                | `vole-module`        | Library       | Module resolver, loader, JS shims         |
| `crates/package`                | `vole-package`       | Library       | Package system                            |
| `crates/cmd`                    | `vole-cli`           | Binary `vole` | CLI entrypoint                            |
| `crates/global_bindings/<name>` | `vole-global-<name>` | Library       | Individual global Web API binding crate   |
| `crates/raw_bindings/<name>`    | `vole-raw-<name>`    | Library       | Individual `vole:raw:*` backend crate     |

---

## Build & Check

```sh
cargo check                          # quick validation
cargo build                          # full build
cargo build -p vole-cli              # build only the CLI binary
./target/debug/vole --help           # check CLI
```

---

## CLI Commands

```
vole <file>              # Execute a JavaScript file
vole run <file>          # Explicit run subcommand
vole --watch <file>      # Execute with file watching
vole eval <code>         # Evaluate JavaScript code
vole repl                # Start interactive REPL
vole test [path]         # Run *_test.js / *_test.ts files recursively
vole -v                  # Print vole version
vole --version           # Print vole version
vole version             # Print version including key dependencies
vole help [command]      # Print help
vole init [name] [dir]   # Initialize a new project (package.json + main.js)
vole install [pkg...]    # Install dependencies from npm registry
```

---

## Tests

Test files use the `*_test.js` / `*_test.ts` naming convention and are kept in the `tests/` directory.
Run all tests with:

```sh
vole test .       # recursively run tests/
vole test tests/  # run a specific directory
vole test tests/foo_test.js  # run a single file
```

`tests/external/` holds compatibility tests adapted from other runtimes
(notably AWS LLRT). See `tests/external/README.md` for sources, status, and
known runtime differences.

---

## Key Dependencies

- `rquickjs = "0.12"` with features `futures`, `loader` — QuickJS async bindings
- `tokio` — async runtime (both crates)
- `clap` with `derive` — CLI argument parsing
- `oxc_*` — TypeScript transpilation (`crates/modules`)
- `reqwest` / `axum` — HTTP client / server
- `wasmi` — WebAssembly interpreter
- `libffi` / `libloading` — FFI
- `rsa` / `aes` / `sha*` — Crypto

---

## Architecture Notes

- `Vm` wraps `rquickjs::AsyncRuntime` + `rquickjs::AsyncContext`. All `Vm` methods are async and require a tokio runtime.
- `Runtime` is the legacy synchronous wrapper (stateless, one context per `eval`). Prefer `Vm` for new code.
- Built-in modules use `"vole:"` prefix (e.g. `vole:console`, `vole:fs`, `vole:net/http`).
- JS shim files in `crates/modules/src/js_shims/` provide the JS API for built-in modules, delegating to `globalThis.__*` Rust functions.
- rquickjs `IntoJs<'js>` lifetime bounds prevent generic `set_global`; type-specific setters (`set_global_string`, `set_global_number`, `set_global_bool`) are used instead.
- `with_context` callback must return `Result<R, rquickjs::Error>` (not `VmError`).

---

## Conventions

- Edition 2021.
- Error types use `thiserror`.
- Module system rejects bare specifiers (e.g. `"lodash"`) to prevent ghost dependencies.
- Web API implementations live in `crates/global_bindings/<name>/src/lib.rs` as `vole-global-<name>` crates.
- System-capability bindings live in `crates/raw_bindings/<name>/src/lib.rs` as `vole-raw-<name>` crates and are exposed via `vole:raw:*` native modules.

---

## Built-in Modules (`vole:*`)

| Module  | Prefix       | Status      | Notes                                                    |
| ------- | ------------ | ----------- | -------------------------------------------------------- |
| env     | vole:env     | Implemented |                                                          |
| ffi     | vole:ffi     | Implemented | Dynamic library calls via libffi/libloading              |
| fs      | vole:fs      | Implemented | Path-based sync/async + file-handle open/read/write/seek |
| http    | vole:net/http    | Implemented | Client request + axum server with JS handler callbacks   |
| path    | vole:path    | Implemented | Cross-platform path helpers                              |
| process | vole:process | Implemented | compatibility wrapper；底层能力来自 `vole:raw:os` 系列 |
| shell   | vole:shell   | Implemented | `exec()` / `execSync()`                                  |
| socket  | vole:net/socket  | Implemented | Unix domain socket (Unix only)                           |
| spawn   | vole:spawn   | Implemented | `spawn()` child process                                  |
| tcp     | vole:net/tcp     | Implemented | `connect()` / `listen()` / `Server`                      |
| test    | vole:test    | Implemented | Built-in test framework used by `vole test`              |
| udp     | vole:net/udp     | Implemented | `createSocket()`                                         |
| wasm    | vole:wasm    | Implemented | WebAssembly loader helpers                               |

---

## Web API Bindings

> 立场：vole 是**纯粹的 ECMAScript 运行时**，不是浏览器。Web API 以 WinterTC Minimum Common Web Platform API 为参考，但会对浏览器、DOM、HTML、CSS 专属概念进行甄别和替代；无法替代的不予实现。

Located in `crates/global_bindings/`. Registered via `ExtensionRegistry` builder in `crates/runtime/src/bindings.rs`.

- **Phase 0**: console, timers, performance, base64, DOMException, reportError
- **Phase 1**: TextEncoder/TextDecoder, URL/URLSearchParams, Blob/File, CompressionStream/DecompressionStream, structuredClone
- **Phase 2**: Event/EventTarget/CustomEvent, MessageEvent/MessageChannel/MessagePort, AbortController/AbortSignal
  - `EventTarget` is implemented as a Rust native class; `globalThis` is wired to behave as an `EventTarget` instance.
  - Runtime errors and unhandled Promise rejections are dispatched as `"error"` / `"unhandledrejection"` / `"rejectionhandled"` events on `globalThis`.
- **Phase 3**: fetch/Request/Response/Headers/FormData
- **Phase 4**: Web Crypto (`crypto`, `CryptoKey`, `SubtleCrypto`)
- **Streams**: ReadableStream/WritableStream/TransformStream and readers/writers/controllers
- **Encoding streams**: TextEncoderStream/TextDecoderStream
- **Compression**: `CompressionStream` / `DecompressionStream` are global bindings defined in `global_bindings/compression.rs`; no `vole:compression` shim module
- **Performance**: `Performance` constructor exposed; `performance` is an instance of `Performance`
- **Blob**: `Blob.prototype.stream()` returns a real `ReadableStream`
- **WebAssembly**: global `WebAssembly` object based on `wasmi` (partial: missing `Global` / `Table` / `Tag` / `Exception` / `LinkError` and streaming API)

> 详细状态（含缺失接口列表）见 `ROADMAP.md` 第一章。

`env`, `process`, `fs`, `path` are also exposed as low-level globals for the corresponding `vole:*` modules.

---

## Package System

`crates/package` provides:

- `PackageManager::init_project(name, dir)` — creates `package.json` + `main.js`
- `PackageManager::install_packages(packages, dir)` — downloads and extracts npm tarballs to `node_modules`
- `PackageManager::install_all(dir)` — installs dependencies from existing `package.json`

CLI commands: `vole init` / `vole install`.

---

## Roadmap & Detailed API Docs

- `ROADMAP.md` — implementation roadmap and priority list (source of truth for status)
- `API.md` — detailed API design and planned interfaces
- `ECOSYSTEM.md` — rquickjs ecosystem notes and AWS LLRT reference patterns
- `WinterTC.md` — WinterTC Minimum Common Web Platform API alignment tracking

---

## Git Commit Convention

提交信息遵循 **Conventional Commits** 风格，便于生成 changelog 和自动化版本管理。

### 格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

- `subject`：简短描述（不超过 72 字符），句末不加句号。
- `body`：可选，详细说明改动动机、影响范围、 breaking changes。
- `footer`：可选，用于关联 issue / PR / breaking change 标记。

### Type

| Type       | 含义                                           |
| ---------- | ---------------------------------------------- |
| `feat`     | 新功能                                         |
| `fix`      | Bug 修复                                       |
| `docs`     | 文档-only 改动                                 |
| `style`    | 不影响代码含义的格式改动（空格、分号、格式化） |
| `refactor` | 重构（既不是 feat 也不是 fix）                 |
| `perf`     | 性能优化                                       |
| `test`     | 测试相关                                       |
| `chore`    | 构建、工具、依赖、CI 等杂项                    |
| `revert`   | 回滚提交                                       |

### Scope

| Scope     | 说明                                                                   |
| --------- | ---------------------------------------------------------------------- |
| `core`    | `crates/core`（vole-core）                                              |
| `runtime` | `crates/runtime`（vole-runtime）                                       |
| `modules` | `crates/modules`（vole-module）                                        |
| `package` | `crates/package`（vole-package）                                       |
| `cli`     | `crates/cmd`（vole-cli）                                               |
| `tests`   | `tests/` 目录                                                          |
| `docs`    | `AGENTS.md` / `ROADMAP.md` / `API.md` / `WinterTC.md` / `README.md` 等 |
| `root`    | 仓库根配置（`.gitignore`、`Cargo.toml` 等）                            |
| `bench`   | `benchmarks/`                                                          |

### 示例

```
feat(cli): add `vole help [command]` subcommand

Replace clap's auto-generated help with an explicit `Help` subcommand
so that `vole help run` prints command-specific usage.

docs(root): add Git commit convention to AGENTS.md

Document Conventional Commits type/scope rules used across the workspace.
```

### 初始提交组织建议

首次提交建议按模块/类型分批，便于后续回溯：

1. `chore(root): bootstrap workspace with Cargo.toml and .gitignore`
2. `docs(root): add README, AGENTS, ROADMAP, API, ECOSYSTEM, WinterTC docs`
3. `feat(core): implement async VM and Web API bindings`
4. `feat(modules): implement module resolver, loader, JS shims and TS transpile`
5. `feat(package): implement project init and npm install`
6. `feat(cli): implement vole CLI commands`
7. `test(root): add test suite and external compatibility tests`
8. `chore(bench): add benchmarks`
