# WinterTC Minimum Common Web Platform API 对齐追踪

> 追踪 vole runtime 对 **WinterTC Minimum Common Web Platform API 2025 snapshot** 的对齐情况。
>
> 官方规范：[https://min-common-api.proposal.wintertc.org/](https://min-common-api.proposal.wintertc.org/)
>
> **重要立场**：vole 是一个**纯粹的 ECMAScript 运行时**，不是浏览器。我们尽可能兼容 WinterTC 与 TC39 的公共运行时接口，但会对其中属于浏览器、DOM、HTML、CSS 的概念进行甄别：能用运行时概念替代的予以替代，无法替代的则明确抛弃。因此下表中某些 WinterTC 接口会被标记为“🔁 运行时替代”或“➖ 不适用/主动不实现”，这不代表实现遗漏，而是代表它们不属于 vole 的浏览器模型。
>
> **文档定位**：`ROADMAP.md` 是实现路线和优先级的来源；本文档只维护 WinterTC 对齐口径。实现状态变更时，应先校准 `ROADMAP.md`，再同步更新本文档。

---

## 图例

| 标记 | 含义                                                             |
| :--: | ---------------------------------------------------------------- |
|  ✅  | 已按 vole 运行时目标完整实现；不等同于浏览器完整规范覆盖         |
|  ⚠️  | 已实现核心能力，但存在明确限制、兼容性偏差或缺少规范关键语义     |
|  ❌  | 未实现，且属于 vole 运行时目标范围                               |
|  🔁  | 已用 vole 运行时概念替代；不是原接口实现，但已有等价运行时路径   |
|  ➖  | 主动不实现 / 不适用；不计入待实现缺口                            |

### 状态口径

- 本文档追踪的是 **vole 对 WinterTC 的运行时适配结果**，不是浏览器兼容性认证。
- `✅` 必须代表当前实现满足 vole 定义的目标语义，并具备基础测试覆盖。
- `⚠️` 用于保留已可用但有已知规范缺口的能力，避免把“可用”误写成“完整”。
- `🔁` 用于记录“概念被替代”的场景，例如用 `globalThis` 事件、`vole:os` / `vole:*` 模块承担浏览器接口职责。
- `➖` 是主动产品取舍或确实不适用，不应在汇总和路线图中被当作待补齐任务。

---

## 1. Common Interfaces

WinterTC snapshot 列出以下 53 个 Common Interfaces。vole 按运行时模型将其分为已实现、部分实现、待实现、运行时替代和主动不适用五类。

### 1.1 DOM

| Interface         | Status | Notes |
| ----------------- | :----: | ----- |
| `AbortController` |   ✅   | —     |
| `AbortSignal`     |   ✅   | —     |
| `Event`           |   ✅   | —     |
| `EventTarget`     |   ✅   | —     |

### 1.2 HTML

| Interface               | Status | Notes                                                                                                  |
| ----------------------- | :----: | ------------------------------------------------------------------------------------------------------ |
| `CustomEvent`           |   ✅   | —                                                                                                      |
| `ErrorEvent`            |   🔁   | HTML 错误事件类型；已用 `globalThis` 的 `"error"` 事件替代，`reportError()` 与未捕获异常均会派发该事件 |
| `MessageChannel`        |   ✅   | —                                                                                                      |
| `MessageEvent`          |   ✅   | —                                                                                                      |
| `MessagePort`           |   ⚠️   | 已实现，但仅能在同一 VM 内通信；无 Worker / 跨线程能力                                                 |
| `PromiseRejectionEvent` |   🔁   | HTML Promise 拒绝事件类型；已用 `globalThis` 的 `"unhandledrejection"` / `"rejectionhandled"` 事件替代 |

### 1.3 WEBIDL

| Interface      | Status | Notes                                                                                     |
| -------------- | :----: | ----------------------------------------------------------------------------------------- |
| `DOMException` |   ✅   | DOM 标准错误类型；保留作兼容性别名，未来如定义运行时通用 `Exception` 可考虑以别名为其暴露 |

### 1.4 Fetch

| Interface  | Status | Notes                                                         |
| ---------- | :----: | ------------------------------------------------------------- |
| `Headers`  |   ✅   | 大小写不敏感键                                                |
| `Request`  |   ✅   | 含 mode/credentials/cache/redirect/referrer/integrity/clone() |
| `Response` |   ✅   | text/json/arrayBuffer/blob + error()/redirect() 静态方法      |

### 1.5 XHR

| Interface  | Status | Notes                                                        |
| ---------- | :----: | ------------------------------------------------------------ |
| `FormData` |   ✅   | append/delete/get/getAll/has/set/entries/keys/values/forEach |

### 1.6 File API

| Interface | Status | Notes                                                                                      |
| --------- | :----: | ------------------------------------------------------------------------------------------ |
| `Blob`    |   ✅   | text/arrayBuffer/slice/bytes/stream；`stream()` 返回真正 `ReadableStream`                  |
| `File`    |   ✅   | 继承 Blob + name/lastModified/webkitRelativePath；保留以兼容 fetch FormData / 文件上传场景 |

### 1.7 Compression

| Interface             | Status | Notes              |
| --------------------- | :----: | ------------------ |
| `CompressionStream`   |   ✅   | 已作为全局对象暴露 |
| `DecompressionStream` |   ✅   | 已作为全局对象暴露 |

### 1.8 Streams

| Interface                          | Status | Notes                                            |
| ---------------------------------- | :----: | ------------------------------------------------ |
| `ByteLengthQueuingStrategy`        |   ✅   | 简单策略对象                                     |
| `CountQueuingStrategy`             |   ✅   | 简单策略对象                                     |
| `ReadableByteStreamController`     |   ❌   | 字节流 controller                                |
| `ReadableStream`                   |   ✅   | 构造函数 + getReader() + locked，async pull 背压 |
| `ReadableStreamBYOBReader`         |   ❌   | 字节流 reader                                    |
| `ReadableStreamBYOBRequest`        |   ❌   | 字节流请求对象                                   |
| `ReadableStreamDefaultController`  |   ✅   | enqueue()/close()/error()/desiredSize            |
| `ReadableStreamDefaultReader`      |   ✅   | read() Promise + releaseLock() + closed          |
| `TransformStream`                  |   ✅   | 构造函数 + readable/writable                     |
| `TransformStreamDefaultController` |   ✅   | enqueue()/error()/terminate()                    |
| `WritableStream`                   |   ✅   | 构造函数 + getWriter() + locked                  |
| `WritableStreamDefaultController`  |   ✅   | error()                                          |
| `WritableStreamDefaultWriter`      |   ✅   | write()/close()/abort() + releaseLock() + closed |

### 1.9 Encoding

| Interface           | Status | Notes                           |
| ------------------- | :----: | ------------------------------- |
| `TextDecoder`       |   ✅   | decode/encoding/fatal/ignoreBOM |
| `TextDecoderStream` |   ✅   | 基于 TransformStream 实现       |
| `TextEncoder`       |   ✅   | encode/encodeInto/encoding      |
| `TextEncoderStream` |   ✅   | 基于 TransformStream 实现       |

### 1.10 URL

| Interface         | Status | Notes                                  |
| ----------------- | :----: | -------------------------------------- |
| `URL`             |   ✅   | 所有属性 + canParse()/parse()/toJSON() |
| `URLSearchParams` |   ✅   | 完整迭代器 + size/sort                 |

### 1.11 URL Pattern

| Interface    | Status | Notes                              |
| ------------ | :----: | ---------------------------------- |
| `URLPattern` |   ⚠️   | 基于正则的 JS shim，非完整规范实现 |

### 1.12 Web Crypto

| Interface      | Status | Notes                                    |
| -------------- | :----: | ---------------------------------------- |
| `Crypto`       |   ✅   | 空构造函数已暴露                         |
| `CryptoKey`    |   ✅   | key handle 包装，支持 raw/jwk/pkcs8/spki |
| `SubtleCrypto` |   ✅   | 已暴露为 SubtleCrypto 实例               |

### 1.13 High-Resolution Time

| Interface     | Status | Notes                                                                                              |
| ------------- | :----: | -------------------------------------------------------------------------------------------------- |
| `Performance` |   ✅   | `performance` 全局对象 + `Performance` 构造函数均已暴露；`performance instanceof Performance` 为真 |

### 1.14 WebAssembly

| Interface                  | Status | Notes                                         |
| -------------------------- | :----: | --------------------------------------------- |
| `WebAssembly.Global`       |   ❌   | WASM 全局对象                                 |
| `WebAssembly.Instance`     |   ✅   | 函数调用 + 内存读写 + import 对象支持 JS 回调 |
| `WebAssembly.Memory`       |   ✅   | 内存读写                                      |
| `WebAssembly.Module`       |   ✅   | compile/validate                              |
| `WebAssembly.Table`        |   ❌   | WASM 表                                       |
| `WebAssembly.Tag`          |   ❌   | 异常标签                                      |
| `WebAssembly.Exception`    |   ❌   | 异常对象                                      |
| `WebAssembly.CompileError` |   ✅   | —                                             |
| `WebAssembly.LinkError`    |   ❌   | 链接错误类型                                  |
| `WebAssembly.RuntimeError` |   ✅   | —                                             |

---

## 2. Common Methods and Properties

WinterTC snapshot 列出以下 Common Methods and Properties。vole 按运行时模型将其分为已实现、部分实现、待实现、运行时替代和主动不适用五类。

### 2.1 HTML

| API                    | Status | Notes                                                                                                            |
| ---------------------- | :----: | ---------------------------------------------------------------------------------------------------------------- |
| `globalThis`           |   ✅   | —                                                                                                                |
| `atob()`               |   ✅   | —                                                                                                                |
| `btoa()`               |   ✅   | —                                                                                                                |
| `clearTimeout()`       |   ✅   | —                                                                                                                |
| `clearInterval()`      |   ✅   | —                                                                                                                |
| `navigator.userAgent`  |   🔁   | 浏览器 `navigator` 概念；vole 已移除 `navigator`，运行时信息通过 `vole:os`（进程/运行时信息）提供 |
| `onerror`              |   🔁   | HTML `Window` / `WorkerGlobalScope` 事件处理器；vole 不实现该属性，错误通过 `"error"` 事件在 `globalThis` 上派发 |
| `onunhandledrejection` |   🔁   | HTML 属性；已用 `globalThis` 的 `"unhandledrejection"` 事件替代                                                  |
| `onrejectionhandled`   |   🔁   | HTML 属性；已用 `globalThis` 的 `"rejectionhandled"` 事件替代                                                    |
| `queueMicrotask()`     |   ✅   | —                                                                                                                |
| `reportError()`        |   ✅   | 派发 `"error"` 事件；若未被取消默认行为则回退打印                                                                |
| `self`                 |   🔁   | 浏览器 / Worker 全局别名概念；vole 已移除 `self`，统一使用 `globalThis`                                          |
| `setTimeout()`         |   ✅   | 含 args 传递                                                                                                     |
| `setInterval()`        |   ✅   | 含 args 传递                                                                                                     |
| `structuredClone()`    |   ⚠️   | serde_json 往返；仅适合 JSON-serializable 数据，无 transfer list / 循环引用 / Map / Set / Date 等完整语义        |

### 2.2 Fetch

| API       | Status | Notes                 |
| --------- | :----: | --------------------- |
| `fetch()` |   ✅   | GET/POST/Headers/body |

### 2.3 Console

| API       | Status | Notes                                      |
| --------- | :----: | ------------------------------------------ |
| `console` |   ✅   | log/info/warn/error/debug/time/timeEnd/... |

### 2.4 Web Crypto

| API      | Status | Notes                                    |
| -------- | :----: | ---------------------------------------- |
| `crypto` |   ✅   | getRandomValues/randomUUID + subtle 全套 |

### 2.5 High-Resolution Time

| API           | Status | Notes                                      |
| ------------- | :----: | ------------------------------------------ |
| `performance` |   ✅   | now/timeOrigin/mark/measure/getEntries/... |

### 2.6 WebAssembly

| API                                  | Status | Notes                                                                                                                                       |
| ------------------------------------ | :----: | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `WebAssembly.compile()`              |   ✅   | —                                                                                                                                           |
| `WebAssembly.compileStreaming()`     |   ❌   | 规范签名依赖浏览器 `Response`，但概念上可脱离浏览器：接受 `ReadableStream` / `Promise<bytes>` / `AsyncIterable<bytes>` 等字节源；待设计实现 |
| `WebAssembly.instantiate()`          |   ✅   | —                                                                                                                                           |
| `WebAssembly.instantiateStreaming()` |   ❌   | 同上；待设计实现                                                                                                                            |
| `WebAssembly.JSTag`                  |   ❌   | WASM JS 异常标签，运行时目标范围                                                                                                            |
| `WebAssembly.validate()`             |   ❌   | 运行时目标范围                                                                                                                              |

---

## 3. Web Workers

WinterTC **不要求**运行时支持 Web Workers。

如果 vole 未来引入 Worker，也会使用**运行时错误/进程机制**处理异常与未捕获拒绝，而不是暴露 HTML 事件模型中的 `onerror` / `onunhandledrejection` / `PromiseRejectionEvent`。

> **注意：`MessagePort` / `MessageChannel`**
>
> 当前实现的 `MessagePort` / `MessageChannel` 仅用于同 VM 内的兼容性占位：消息通过 `setTimeout` 在同一 JavaScript 堆中传递，没有真正的跨上下文能力。
> 一旦 vole 决定支持 Worker，这两者必须重写——最可能改成 Rust native 实现，以支持：
>
> - 跨 Worker / 跨线程的 channel 通信；
> - `postMessage` 的 transferable 语义（如 `ArrayBuffer`、`MessagePort` 的所有权转移）；
> - 跨 VM 的 structured clone 序列化/反序列化；
> - `MessageChannel` 两个端口分别属于不同全局作用域。
>
> 因此现在保留它们只是作为过渡，Worker 实现时是必改项。

---

## 4. 汇总

| 类别                      | 总数 | ✅ 已实现 | ⚠️ 部分实现 | ❌ 未实现 | 🔁 运行时替代 | ➖ 不适用 |
| ------------------------- | :--: | :-------: | :---------: | :-------: | :-----------: | :-------: |
| Common Interfaces         |  53  |    41     |      2      |     8     |       2       |     0     |
| Common Methods/Properties |  25  |    15     |      1      |     4     |       5       |     0     |

> **结论**：vole 不是浏览器，因此不会 100% 复刻 WinterTC 中所有浏览器/DOM/HTML 概念。🔁 项属于已被 vole 运行时机制替代的浏览器接口；➖ 项属于主动不实现或确实不适用的浏览器概念；⚠️ 项是已可用但存在明确限制的兼容层（当前包括 `MessagePort` 的同 VM 占位实现、基于正则的 `URLPattern` shim，以及 JSON-only 的 `structuredClone()`）；❌ 项才是需要在纯运行时模型下继续补齐的能力（主要集中在 Streams 字节流三件套与 WASM 缺失类型与方法）。

---

## 5. 推荐补齐优先级

1. **P0（文档与验证基线）**
   - 对照 `ROADMAP.md`、`tests/external/README.md` 和实际测试结果校准所有 ✅ / ⚠️ 状态
   - 为关键 ✅ 项补充实现位置与测试覆盖引用，避免状态漂移
2. **P1（高）**
   - 补齐 Streams 字节流三件套（`ReadableByteStreamController`、`ReadableStreamBYOBReader`、`ReadableStreamBYOBRequest`）
3. **P2（中）**
   - 补齐 WASM `Global` / `Table` / `Exception` / `LinkError` / `validate()` / `JSTag`
4. **P3（架构级 / 待设计）**
   - `WebAssembly.compileStreaming()` / `instantiateStreaming()`：设计为接受非浏览器字节源（ReadableStream / Promise<bytes> / AsyncIterable）
   - `DOMException`：保留兼容，未来如定义运行时通用 `Exception`，可将 `DOMException` 作为别名暴露
5. **已完成（已从优先级列表移除）**
   - ✅ 让 `globalThis` 实现 `EventTarget`，通过 `"error"` / `"unhandledrejection"` / `"rejectionhandled"` 事件替代 `ErrorEvent` / `PromiseRejectionEvent` / `onerror` 等浏览器概念
   - ✅ 全局暴露 `CompressionStream` / `DecompressionStream`
   - ✅ 暴露 `Performance` 构造函数
   - ✅ 修复 `Blob.stream()` 返回真正 `ReadableStream`
6. **运行时替代 / 已移除浏览器别名**
   - `navigator` 全局对象（含 `navigator.userAgent`）
   - `self` / `window` 全局别名
