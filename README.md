# vole

`vole` 是一个基于 QuickJS / `rquickjs` 构建的 ECMAScript runtime。

项目目标不是复刻 Node.js，也不是成为浏览器，而是提供一个小型、可嵌入、以标准 Web API 为优先设计基础的 JavaScript / TypeScript 运行时。系统能力通过 `vole:*` 内置模块暴露，通用能力尽量使用 WinterTC Minimum Common Web Platform API 对齐。

## 当前定位

- **语言核心**：以 ECMAScript / TC39 语义为基础。
- **通用 API**：优先实现 WinterTC 相关 Web API，例如 `console`、`URL`、`TextEncoder`、`Blob`、`Streams`、`CompressionStream`、`crypto`、`structuredClone`。
- **系统能力**：通过 `vole:*` 模块提供文件、进程、网络、FFI、shell、package 等能力。
- **异步模型**：runtime 基于 Tokio，`Vm` 使用 `rquickjs::AsyncRuntime` 和 `AsyncContext`。
- **非目标**：不直接追求 Node.js API 兼容；Node 风格能力只在确有必要时作为参考，不做适配层。

## 安装与构建

```sh
cargo build
cargo build -p vole-cli
```

构建完成后，CLI 位于：

```sh
./target/debug/vole
```

查看帮助：

```sh
./target/debug/vole --help
./target/debug/vole version
```

## 基本使用

执行 JavaScript / TypeScript 文件：

```sh
vole index.js
vole run index.ts
vole --watch index.ts
```

执行一段代码：

```sh
vole eval "console.log('hello world')"
```

启动 REPL：

```sh
vole repl
```

运行测试：

```sh
vole test .
vole test tests/
vole test tests/url_test.js
```

初始化和安装包：

```sh
vole init my-app
vole install
vole install some-package
```

## Web API

`vole` 的全局 API 以 Web API 为主，而不是 Node.js 全局对象。

已实现或部分实现的主要能力包括：

- `console`
- `setTimeout` / `setInterval` / `setImmediate`
- `performance`
- `atob` / `btoa`
- `DOMException`
- `reportError`
- `TextEncoder` / `TextDecoder`
- `URL` / `URLSearchParams` / `URLPattern`
- `Blob` / `File`
- `ReadableStream` / `WritableStream` / `TransformStream`
- `TextEncoderStream` / `TextDecoderStream`
- `CompressionStream` / `DecompressionStream`
- `Event` / `EventTarget` / `CustomEvent`
- `MessageEvent` / `MessageChannel` / `MessagePort`
- `AbortController` / `AbortSignal`
- `structuredClone`
- `crypto` / `SubtleCrypto`
- `fetch` / `Request` / `Response` / `Headers` / `FormData`
- `WebAssembly`

详细实现状态和缺口见：

- `ROADMAP.md`
- `WinterTC.md`

## `vole:*` 内置模块

`vole:*` 模块用于暴露 runtime 的系统能力。它们不是 Node.js 模块，而是 vole 自己的标准模块命名空间。

常用模块：

- `vole:env`
- `vole:fs`
- `vole:path`
- `vole:process`（对外兼容 API；底层由 `vole:raw:os` 系列实现）
- `vole:shell`
- `vole:spawn`
- `vole:ffi`
- `vole:net/http`
- `vole:net/tcp`
- `vole:net/udp`
- `vole:net/socket`
- `vole:test`
- `vole:wasm`
- `vole:until`

示例：

```js
import { join } from "vole:path";
import { readFileSync } from "vole:fs";

const file = join("data", "hello.txt");
console.log(readFileSync(file, "utf8"));
```

## `vole:until`

`vole:until` 提供 runtime 生命周期延长能力，适合 HTTP request、Worker event、测试收尾、后台日志和指标刷新等场景。

```js
import { WaitUntil } from "vole:until";

const until = new WaitUntil();

until.waitUntil(
  Promise.resolve().then(() => {
    console.log("background work");
  }),
);

await until.drain(30_000);
```

默认 `new WaitUntil()` 会创建 managed scope，并加入 runtime 级 registry。也可以创建 detached scope：

```js
const local = new WaitUntil({ managed: false });
```

## 项目结构

```text
crates/core                   runtime 核心能力、VM 基础设施、共享工具
crates/modules                模块解析、加载器、JS shims
crates/package                package.json / npm package 安装能力
crates/cmd                    vole CLI
crates/global_bindings/*      全局 Web API bindings
crates/raw_bindings/*         vole:raw:* 原生后端模块
tests                         runtime 测试
```

## 开发命令

快速检查：

```sh
cargo check
```

构建 CLI：

```sh
cargo build -p vole-cli
```

运行指定测试：

```sh
cargo run -p vole-cli -- test tests/until_test.js
```

运行全部测试：

```sh
cargo run -p vole-cli -- test tests/
```

## 文档

- `AGENTS.md`：给维护 Agent 的工作区说明、提交约定和架构约定。
- `ROADMAP.md`：当前实现状态和优先级，作为状态来源。
- `API.md`：API 设计和规划。
- `ECOSYSTEM.md`：QuickJS / LLRT / 相关生态参考。
- `WinterTC.md`：WinterTC Minimum Common Web Platform API 对齐追踪。

## 设计原则

- 优先采用标准 Web API。
- 系统能力放入 `vole:*` 命名空间。
- 不默认兼容 Node.js API。
- 能 native first 的 Web API 尽量 native first。
- runtime 核心能力沉淀到 `crates/core`，避免散落在 bindings 中。
- JS shim 只作为 API 组合层，底层能力优先由 Rust native binding 提供。
