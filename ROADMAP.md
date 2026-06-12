# vole-runtime 实施路线图

> 按 WinterTC Minimum Common Web Platform API、标准模块、包系统、CLI 四大章节整理。
> 状态：✅ 已实现 | 📋 计划中（按优先级排序） | ➖ 暂不计划

---

## 〇、TC39 ECMAScript 语言特性

> QuickJS 引擎已原生支持 ES2024 及大部分 ES2025 特性。以下列出**可在运行时层补充**的缺口。

### 引擎已原生支持（无需运行时实现）

| 特性                                                   | 说明                                   |
| ------------------------------------------------------ | -------------------------------------- |
| `Promise.withResolvers()`                              | ES2024                                 |
| `Object.groupBy` / `Map.groupBy`                       | ES2024                                 |
| `Array.prototype.toSorted/toReversed/toSpliced/with`   | ES2023/2024                            |
| `RegExp.escape()`                                      | ES2025                                 |
| `using` / `await using`                                | ES2025（Explicit Resource Management） |
| `ArrayBuffer.prototype.transfer`                       | ES2024                                 |
| Iterator Helpers (`Iterator.prototype.map/filter/...`) | ES2024                                 |
| `Atomics.waitAsync`                                    | —                                      |

### 运行时层可补充

| 特性                                             | 状态 | 计划 | 说明                                            |
| ------------------------------------------------ | :--: | :--: | ----------------------------------------------- |
| JSON modules (`import ... with { type: "..." }`) |  ⚠️  |  P3  | 通用配置导入：json/toml/jsonc/json5/ini/yaml 等 |
| `Temporal` API                                   |  ➖  |  —   | QuickJS 未支持，polyfill 工作量过大，暂不计划   |
| `Float16Array`                                   |  ➖  |  —   | 新型 TypedArray，需求低，暂不计划               |

### 依赖引擎支持（无法 polyfill）

| 特性                 | 说明                               |
| -------------------- | ---------------------------------- |
| Decorators           | 语法层，需 QuickJS 升级 — 暂不计划 |
| Source Phase Imports | 语法层 + 模块系统 — 暂不计划       |
| Pattern Matching     | Stage 1，尚不稳定 — 暂不计划       |

---

## 一、WinterTC Minimum Common Web Platform API

> vole 是**纯粹的 ECMAScript 运行时**，不是浏览器。本章所列 API 以 WinterTC 为参考，但会对浏览器、DOM、HTML、CSS 相关概念进行甄别：能用运行时概念替代的予以替代，无法替代的则明确抛弃。因此某些 WinterTC 接口（如 `ErrorEvent`、`PromiseRejectionEvent`、`onerror` 等）不会实现，而由运行时自身的错误/进程机制覆盖。

### 1.1 Fetch and HTTP — ✅ 完成

| API        | 状态 | 说明                                                          |
| ---------- | :--: | ------------------------------------------------------------- |
| `fetch`    |  ✅  | GET/POST/Headers/body                                         |
| `Request`  |  ✅  | 含 mode/credentials/cache/redirect/referrer/integrity/clone() |
| `Response` |  ✅  | text/json/arrayBuffer/blob + error()/redirect() 静态方法      |
| `Headers`  |  ✅  | 大小写不敏感键，含 getSetCookie()                             |
| `FormData` |  ✅  | append/delete/get/getAll/has/set/entries/keys/values/forEach  |

### 1.2 URL — ⚠️ 基本完成

| API               | 状态 | 说明                                   |
| ----------------- | :--: | -------------------------------------- |
| `URL`             |  ✅  | 所有属性 + canParse()/parse()/toJSON() |
| `URLSearchParams` |  ✅  | 完整迭代器 + size/sort                 |
| `URLPattern`      |  ⚠️  | 基于正则的 JS shim，非完整规范         |

### 1.3 Streams — ⚠️ 部分实现

> 核心 ReadableStream / WritableStream / TransformStream 已可用，但**字节流（BYOB）接口缺失**。

| API                                | 状态 | 计划 | 说明                                             |
| ---------------------------------- | :--: | :--: | ------------------------------------------------ |
| `ReadableStream`                   |  ✅  |  —   | 构造函数 + getReader() + locked，async pull 背压 |
| `ReadableStreamDefaultReader`      |  ✅  |  —   | read() Promise + releaseLock() + closed          |
| `ReadableStreamDefaultController`  |  ✅  |  —   | enqueue()/close()/error()/desiredSize            |
| `WritableStream`                   |  ✅  |  —   | 构造函数 + getWriter() + locked                  |
| `WritableStreamDefaultWriter`      |  ✅  |  —   | write()/close()/abort() + releaseLock() + closed |
| `WritableStreamDefaultController`  |  ✅  |  —   | error()                                          |
| `TransformStream`                  |  ✅  |  —   | 构造函数 + readable/writable                     |
| `TransformStreamDefaultController` |  ✅  |  —   | enqueue()/error()/terminate()                    |
| `ByteLengthQueuingStrategy`        |  ✅  |  —   | 简单策略对象                                     |
| `CountQueuingStrategy`             |  ✅  |  —   | 简单策略对象                                     |
| `ReadableByteStreamController`     |  ❌  |  P1  | 字节流 controller                                |
| `ReadableStreamBYOBReader`         |  ❌  |  P1  | 字节流 reader                                    |
| `ReadableStreamBYOBRequest`        |  ❌  |  P1  | 字节流请求对象                                   |

### 1.4 Encoding — ✅ 基本完成

| API                 | 状态 | 说明                            |                           |
| ------------------- | :--: | ------------------------------- | ------------------------- |
| `TextEncoder`       |  ✅  | encode/encodeInto/encoding      |
| `TextDecoder`       |  ✅  | decode/encoding/fatal/ignoreBOM |
| `TextEncoderStream` |  ✅  | —                               | 基于 TransformStream 实现 |
| `TextDecoderStream` |  ✅  | —                               | 基于 TransformStream 实现 |

### 1.5 Compression — ✅ 完成

> `CompressionStream` / `DecompressionStream` 已作为全局对象暴露，底层基于 `TransformStream` + flate2。

| API                   | 状态 | 计划 | 说明                                                    |
| --------------------- | :--: | :--: | ------------------------------------------------------- |
| `CompressionStream`   |  ✅  |  —   | gzip/deflate/deflate-raw，基于 TransformStream + flate2 |
| `DecompressionStream` |  ✅  |  —   | gzip/deflate/deflate-raw，基于 TransformStream + flate2 |

### 1.6 Crypto — ✅ 基本完成

| API                         | 状态 | 计划 | 说明                                                    |
| --------------------------- | :--: | :--: | ------------------------------------------------------- |
| `crypto`（全局对象）        |  ✅  |  —   | getRandomValues/randomUUID + subtle 全套                |
| `Crypto`（构造函数）        |  ✅  |  —   | 空构造函数已暴露                                        |
| `SubtleCrypto`              |  ✅  |  —   | 已暴露为 SubtleCrypto 实例                              |
| `CryptoKey`                 |  ✅  |  —   | key handle 包装，支持 `raw`/`jwk`/`pkcs8`/`spki`        |
| `crypto.getRandomValues`    |  ✅  |  —   | TypedArray<u8>                                          |
| `crypto.randomUUID`         |  ✅  |  —   | UUID v4                                                 |
| `crypto.subtle.digest`      |  ✅  |  —   | SHA-1/256/384/512，支持 TypedArray/ArrayBuffer/DataView |
| `crypto.subtle.importKey`   |  ✅  |  —   | 支持 `raw`/`jwk`/`pkcs8`/`spki`                         |
| `crypto.subtle.exportKey`   |  ✅  |  —   | 支持 `raw`/`jwk`/`pkcs8`/`spki`                         |
| `crypto.subtle.encrypt`     |  ✅  |  —   | AES-CBC / AES-GCM                                       |
| `crypto.subtle.decrypt`     |  ✅  |  —   | AES-CBC / AES-GCM                                       |
| `crypto.subtle.sign`        |  ✅  |  —   | HMAC-SHA1/256/384/512，RSASSA-PKCS1-v1.5，RSA-PSS       |
| `crypto.subtle.verify`      |  ✅  |  —   | HMAC-SHA1/256/384/512，RSASSA-PKCS1-v1.5，RSA-PSS       |
| `crypto.subtle.generateKey` |  ✅  |  —   | AES(128/192/256) / HMAC / RSASSA-PKCS1-v1.5 / RSA-PSS   |

### 1.7 Binary — ⚠️ 部分实现

| API    | 状态 | 说明                                                                      |
| ------ | :--: | ------------------------------------------------------------------------- |
| `Blob` |  ✅  | text/arrayBuffer/slice/bytes/stream；`stream()` 返回真正 `ReadableStream` |
| `File` |  ✅  | 继承 Blob + name/lastModified/webkitRelativePath                          |

### 1.8 Events — ✅ 事件系统已完成

| API                                               | 状态 | 说明                                                                                                     |
| ------------------------------------------------- | :--: | -------------------------------------------------------------------------------------------------------- |
| `Event` / `EventTarget` / `CustomEvent`           |  ✅  | `EventTarget` 为 Rust native 实现；`globalThis` 已挂载为 `EventTarget`                                   |
| `MessageEvent` / `MessageChannel` / `MessagePort` |  ⚠️  | 消息通道（`MessagePort` 仅同 VM 内可用）                                                                 |
| `AbortController` / `AbortSignal`                 |  ✅  | abort/timeout/any/throwIfAborted                                                                         |
| `ErrorEvent`                                      |  ➖  | HTML 错误事件类型；已通过 `globalThis` 的 `"error"` 事件替代                                             |
| `PromiseRejectionEvent`                           |  ➖  | HTML Promise 拒绝事件类型；已通过 `globalThis` 的 `"unhandledrejection"` / `"rejectionhandled"` 事件替代 |

### 1.9 Timers — ✅ 完成

| API                              | 状态 | 说明         |
| -------------------------------- | :--: | ------------ |
| `setTimeout` / `setInterval`     |  ✅  | 含 args 传递 |
| `clearTimeout` / `clearInterval` |  ✅  | —            |
| `queueMicrotask`                 |  ✅  | —            |

### 1.10 Utilities — ⚠️ 部分实现

| API               | 状态 | 说明                                                                                                                                                                              |
| ----------------- | :--: | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `console`         |  ✅  | log/info/warn/error/debug/time/timeEnd/timeLog/assert/table/count/countReset/dir/group/groupEnd/groupCollapsed/trace/clear/memory/profile/profileEnd/timeStamp/dirxml             |
| `performance`     |  ✅  | now/timeOrigin/mark/measure/clearMarks/clearMeasures/getEntries/getEntriesByType/getEntriesByName/toJSON；`Performance` 构造函数已暴露，`performance instanceof Performance` 为真 |
| `structuredClone` |  ✅  | serde_json 往返，无 transfer list                                                                                                                                                 |
| `atob` / `btoa`   |  ✅  | —                                                                                                                                                                                 |
| `DOMException`    |  ✅  | 含标准 code 映射                                                                                                                                                                  |
| `reportError`     |  ✅  | —                                                                                                                                                                                 |

### 1.11 WebAssembly — ⚠️ 部分实现

| API                                | 状态 | 说明                                          |
| ---------------------------------- | :--: | --------------------------------------------- |
| `WebAssembly.Module`               |  ✅  | compile/validate                              |
| `WebAssembly.Instance`             |  ✅  | 函数调用 + 内存读写 + import 对象支持 JS 回调 |
| `WebAssembly.Memory`               |  ✅  | 内存读写                                      |
| `WebAssembly.CompileError`         |  ✅  | —                                             |
| `WebAssembly.RuntimeError`         |  ✅  | —                                             |
| `WebAssembly.Global`               |  ❌  | WASM 全局对象                                 |
| `WebAssembly.Table`                |  ❌  | WASM 表                                       |
| `WebAssembly.Tag`                  |  ❌  | 异常标签                                      |
| `WebAssembly.Exception`            |  ❌  | 异常对象                                      |
| `WebAssembly.LinkError`            |  ❌  | 链接错误类型                                  |
| `WebAssembly.compileStreaming`     |  ❌  | —                                             |
| `WebAssembly.instantiateStreaming` |  ❌  | —                                             |
| `WebAssembly.JSTag`                |  ❌  | —                                             |
| `WebAssembly.validate`             |  ❌  | —                                             |

> **WinterTC 对齐小结**：
>
> - 53 个 Common Interfaces 中，40 个已完整实现，8 个缺失，2 个部分实现（URLPattern、MessagePort）。
> - Common Methods/Properties 基本覆盖；WASM streaming API (`compileStreaming`/`instantiateStreaming`/`validate`/`JSTag`) 仍缺失。
> - 下一阶段重点：补齐 Streams 字节流三件套、补齐 WASM 缺失接口。

---

## 二、标准模块 `std:`

### 2.1 已实现模块

| 模块          | 状态 | 说明                                                                                                 |
| ------------- | :--: | ---------------------------------------------------------------------------------------------------- |
| `std:fs`      |  ✅  | 同步/异步：readFile/readTextFile/writeFile/exists/mkdir/readdir/stat/remove/rename/copyFile/realPath |
| `std:path`    |  ✅  | join/resolve/dirname/basename/extname/isAbsolute/relative/parse/format                               |
| `std:env`     |  ✅  | get/set/has/delete/getAll                                                                            |
| `std:process` |  ✅  | exit/abort/cwd/argv/stdout/stderr/stdin/pid/platform                                                 |
| `std:test`    |  ✅  | test/describe/it/beforeAll/afterAll/beforeEach/afterEach/run + CLI `vole test`                       |
| `std:http`    |  ✅  | client: request/fetch；server: serve/Server（axum + JS handler 回调）                                |

### 2.2 其他模块

| 模块                | 状态 | 计划 | 说明                                                                       |
| ------------------- | :--: | :--: | -------------------------------------------------------------------------- |
| `std:tcp`           |  ✅  |  —   | `connect()` / `listen()` / `Server` — 用 tokio::net::TcpStream/TcpListener |
| `std:udp`           |  ✅  |  —   | `createSocket()` — 用 tokio::net::UdpSocket                                |
| `std:socket` (Unix) |  ✅  |  —   | Unix domain socket — 用 tokio::net::UnixStream（Unix only）                |
| `std:shell`         |  ✅  |  —   | `exec()` / `execSync()` — 基于 process.rs 实现                             |
| `std:spawn`         |  ✅  |  —   | `spawn()` — 基于 process.rs 实现                                           |
| `std:file`          |  ✅  |  —   | `open()` — 文件句柄/seek/read/write                                        |
| `std:ffi`           |  ✅  |  —   | 动态库加载、C ABI 调用，支持整数/浮点/指针/字符串类型                      |
| `std:tracing`       |  ✅  |  —   | 日志函数（info/warn/error/debug/trace），Phase 5 已完成                    |

### 2.3 已注册但未绑定底层函数的模块

> `std:udp` / `std:socket` 已实现底层绑定。
> `std:ffi` 已实现底层绑定。

---

## 三、包系统

> `crates/package` 已实现核心能力，`vole init` / `vole install` 已可用。

| 功能                 | 状态 | 计划 | 说明                                      |
| -------------------- | :--: | :--: | ----------------------------------------- |
| `vole-package` crate |  ✅  |  —   | 包管理核心库（init/install）              |
| `package.json` 解析  |  ✅  |  —   | 依赖/版本解析                             |
| 远程包下载           |  ✅  |  —   | 从 npm registry 拉取 tarball              |
| 本地包缓存           |  ⚠️  |  P4  | 解压到 `node_modules`，暂无独立全局缓存   |
| `import maps` 支持   |  ✅  |  —   | `importmap.json`：精确匹配 + 前缀匹配     |
| 语义版本解析         |  ✅  |  —   | semver 兼容                               |
| CLI `vole init`      |  ✅  |  —   | 初始化项目模板（package.json + 入口文件） |
| CLI `vole install`   |  ✅  |  —   | 安装依赖命令                              |
| CLI `vole publish`   |  ➖  |  —   | 远期                                      |

---

## 四、CLI (`vole`)

### 4.1 已实现

| 命令                         | 状态 | 说明                                    |
| ---------------------------- | :--: | --------------------------------------- |
| `vole <file>`                |  ✅  | 默认执行文件                            |
| `vole eval <code>`           |  ✅  | 直接执行代码                            |
| `vole repl`                  |  ✅  | 交互式 REPL                             |
| `vole -v` / `vole --version` |  ✅  | 打印 vole 版本                          |
| `vole version`               |  ✅  | 打印版本及关键依赖版本                  |
| `vole test [path]`           |  ✅  | 递归运行 `*_test.js` / `*_test.ts` 文件 |
| `vole --watch <file>`        |  ✅  | 文件变更自动重启                        |
| `vole help [command]`        |  ✅  | 打印帮助信息                            |

### 4.2 计划中

| 命令/功能                     | 状态 | 计划 | 说明                                      |
| ----------------------------- | :--: | :--: | ----------------------------------------- |
| `vole run <file>`             |  ✅  |  —   | 显式 `run` 子命令（与默认行为一致）       |
| `vole init`                   |  ✅  |  —   | 初始化项目模板（package.json + 入口文件） |
| `vole install`                |  ✅  |  —   | 安装依赖（依赖包系统）                    |
| `vole fmt`                    |  ➖  |  —   | 代码格式化，远期                          |
| `vole lint`                   |  ➖  |  —   | 代码检查，远期                            |
| `vole bundle`                 |  ➖  |  —   | 打包，远期                                |
| `--inspect` / `--inspect-brk` |  ➖  |  —   | 调试协议，远期                            |
| TypeScript 直接执行           |  ✅  |  —   | `vole run index.ts` — 集成 oxc 转译       |
| `.env` 自动加载               |  ✅  |  —   | 启动时自动加载 `.env` 文件到 `std:env`    |
| `--allow-*` 权限标记          |  ➖  |  —   | Deno 风格的权限系统，远期                 |

---

## 五、执行优先级（推荐顺序）

```
P1 (当前冲刺)
  ├─ 继续扩展外部测试集（Node/Deno/Bun 通用用例）
  └─ std:ffi 高级能力（Pointer 内存读写 / struct 布局 / 回调）

P2 (下一冲刺)
  ├─ 包系统增强（独立全局缓存、私有 registry）
  └─ CLI 生态（`vole fmt` / `vole lint` / `vole bundle`）

P3 (已完成 ✅)
  ├─ ReadableStream + fetch Response.body 改造
  ├─ ReadableStreamDefaultReader + Controller
  ├─ WritableStream + WritableStreamDefaultWriter + Controller
  ├─ TransformStream
  ├─ std:tcp (connect/listen/Server)
  ├─ std:file (open/seek/read/write)
  ├─ CryptoKey + importKey/exportKey (raw/jwk/pkcs8/spki)
  ├─ SubtleCrypto 对象暴露
  ├─ JSON modules / 通用配置导入 (`import ... with { type: "..." }`)
  ├─ TypeScript 直接执行（oxc 转译）
  ├─ CLI `.env` 自动加载
  ├─ `vole run` 显式子命令
  └─ std:shell (exec) + std:spawn

P4 (已完成 ✅)
  ├─ CompressionStream / DecompressionStream
  ├─ std:udp + std:socket (Unix)
  ├─ TextEncoderStream / TextDecoderStream
  ├─ ByteLengthQueuingStrategy / CountQueuingStrategy
  ├─ Crypto RSA (`RSASSA-PKCS1-v1.5` / `RSA-PSS`) pkcs8/spki sign/verify
  └─ import maps

P5 (已完成 ✅)
  ├─ WebAssembly 暴露（Module/Instance/Memory/函数调用/内存读写）
  ├─ WebAssembly import 对象支持 JS 函数回调
  └─ std:tracing

P6+ (远期)
  ├─ CLI fmt/lint/bundle
  └─ --inspect 调试
```

---

_本路线图随项目迭代持续更新。_
