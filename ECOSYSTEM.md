# rquickjs 生态与 AWS LLRT 源码参考

> 本文档汇总 rquickjs 生态可用 crate 及 AWS LLRT 各模块的实现模式，供 vole runtime 开发参考。

---

## 一、可直接引入的 crate

| crate                | 用途                                      | 状态   |
| -------------------- | ----------------------------------------- | ------ |
| `rquickjs` `0.6`     | 核心绑定（已引入）                        | ✅     |
| `rquickjs-extension` | 扩展注册框架，替代手写 `globals.set(...)` | 待引入 |
| `rquickjs-serde`     | Serde 序列化桥接，Rust ↔ JS 值自动转换    | 待引入 |

---

## 二、AWS LLRT 模块分析摘要

LLRT 源码路径：`/Users/bourdon/dev/llrt-reference/`

### 2.1 llrt_console — console 实现

**核心模式：**

- `ctx.store_userdata::<ConsoleState>()` 在 Ctx 上挂载计时器/计数器状态
- `ModuleDef` + `init()` 双重注册：既作为 ESM 模块导出，又直接挂到 `globalThis.console`
- `llrt_logging::build_formatted_string` 统一处理 `%s` `%d` `%j` `%O` 格式化、颜色、循环引用检测

**参考点：**

- `console.time/timeEnd` 用 `HashMap<String, Instant>` 实现
- `log`/`info` → stdout，`warn`/`error` → stderr，`debug`/`trace` → stdout

### 2.2 llrt_timers — setTimeout / setInterval

**核心模式：**

- **单 tokio task 集中调度**，非每个 timer 一个 task
- 全局 `Lazy<Mutex<Vec<RuntimeTimerState>>>` 按 `JSRuntime` 分桶
- `Persistent<Function>` 持久化 JS 回调，到期后 `restore(&ctx)` 恢复调用
- `tokio::time::sleep_until(deadline) + Notify` 高效等待与唤醒
- 每次 JS 回调前后执行 `ctx.execute_pending_job()` 清空微任务

**关键代码：**

```rust
static RT_TIMER_STATE: Lazy<Mutex<Vec<RuntimeTimerState>>> = ...;

// 注册 timer
let callback = Persistent::<Function>::save(ctx, cb);
let timeout = Timeout { deadline, callback: Some(callback), raw_ctx, id, repeating, interval };

// 执行回调
let timeout = timeout.restore(&ctx2)?;
timeout.call::<_, ()>(())?;
```

### 2.3 llrt_fetch — fetch / Request / Response / Headers

**核心模式：**

- HTTP 引擎：`hyper` 1.x（非 reqwest），`tokio::sync::Semaphore(500)` 限制并发
- `#[rquickjs::class]` + `#[rquickjs::methods(rename_all = "camelCase")]` 宏暴露 Rust 结构体为 JS 类
- `Promise::wrap_future(&ctx, async { ... })` 将 Rust Future 转 JS Promise
- `Response.body` 懒加载：首次访问时将 `hyper::Incoming` 包装为 `ReadableStream`
- `AbortSignal` 集成：`tokio::select!` 监听取消信号

**参考点：**

- `Request` / `Response` / `Headers` 均用 `rquickjs::class` 宏封装
- Body 管理：`enum BodyVariant { Incoming(...), Provided(...), Empty }`
- `HeadersGuard` 实现不同上下文下的头部校验策略

### 2.4 llrt_url — URL / URLSearchParams

**核心模式：**

- 底层：`url` crate（servo/rust-url）
- `Rc<RefCell<Url>>` 共享可变状态：URL 和 URLSearchParams 双向同步
- 大量 WHATWG 合规性补丁（file URL host、Windows 驱动器盘符、`/./` sentinel 等）

### 2.5 llrt_crypto — Web Crypto API

**核心模式：**

- `CryptoProvider` trait 抽象多后端：
  - `crypto-rust`（默认）：`aes`、`aes-gcm`、`sha2`、`rsa`、`p256`、`ed25519-dalek` 等
  - `crypto-ring`：仅 Digest/HMAC
  - `crypto-openssl`：全功能
- `#[rquickjs::class]` 封装 `CryptoKey`：
  - `handle: Rc<[u8]>` 持有密钥原始字节
  - `check_validity(usage)` 每次操作前校验权限

**关键代码：**

```rust
#[rquickjs::class]
pub struct CryptoKey<'js> {
    pub kind: KeyKind,
    pub extractable: bool,
    pub algorithm: KeyAlgorithm,
    pub usages: Vec<String>,
    pub handle: Rc<[u8]>,
}
```

### 2.6 llrt_fs — 文件系统

**核心模式：**

- 异步：`tokio::fs`，同步：`std::fs`
- `Func::from(Async(read_file))` vs `Func::from(read_file_sync)` 区分异步/同步
- 共用辅助函数（如 `handle_read_file_bytes`）处理 options（encoding）和后处理
- `#[rquickjs::class]` 封装 `Stats`、`Dirent`、`FileHandle`
- `#[cfg(unix)]` / `#[cfg(windows)]` 条件编译处理平台差异字段（mode、uid、gid 等）

### 2.7 llrt_events — Event / EventTarget / EventEmitter

**核心模式：**

- `Emitter` trait 同时驱动 Node.js `EventEmitter` 和 DOM `EventTarget`
- 存储：`Arc<RwLock<Vec<(EventKey, Vec<EventItem>)>>>`
- `EventKey` 支持 Symbol 和 String
- `dispatchEvent` 触发时 clone 所有 callback，再用 `retain` 移除 `once` 监听器
- 支持 `defer` 模式（宏任务延迟执行）
- 通过隐藏 Symbol `__ee` 将事件能力动态附加到任意 JS 对象

### 2.8 llrt_stream_web — Web Streams API

**核心模式：**

- 严格遵循 WHATWG Streams Standard，复杂状态机
- `ReadableStream`：三态 `Readable / Closed / Errored`
- `WritableStream`：四态 `Writable / Closed / Erroring / Errored`
- 背压：`highWaterMark - queueTotalSize`，`writer.ready` Promise 追踪背压状态
- `QueueWithSizes`：统一队列抽象，`VecDeque<ValueWithSize>`
- `ResolveablePromise`：内部可手动 resolve/reject 的 Promise，用于 `writer.ready`、`write()` 返回值
- `TransformStream`：组合内部 `ReadableStream` + `WritableStream`，双向背压协调

---

## 三、对我们项目最有价值的参考

| 我们的 API                       | LLRT 参考模块     | 学习重点                                                    |
| -------------------------------- | ----------------- | ----------------------------------------------------------- |
| `console` 补全                   | `llrt_console`    | `ConsoleState` userdata 模式、格式化输出                    |
| `setTimeout` / `setInterval`     | `llrt_timers`     | **单 task 集中调度**、Persistent 回调、sleep_until + Notify |
| `fetch` / `Request` / `Response` | `llrt_fetch`      | `rquickjs::class` 宏封装、Promise::wrap_future、Body 懒加载 |
| `URL` / `URLSearchParams`        | `llrt_url`        | `url` crate 使用、Rc<RefCell<Url>> 共享状态                 |
| `crypto.subtle.*`                | `llrt_crypto`     | `CryptoProvider` trait 多后端、`CryptoKey` 封装             |
| `vole:fs`                         | `llrt_fs`         | `tokio::fs` 桥接、同步/异步复用、`Stats` 类封装             |
| `Event` / `EventTarget`          | `llrt_events`     | `Emitter` trait、事件存储模型、defer 模式                   |
| `ReadableStream`（远期）         | `llrt_stream_web` | 状态机、背压控制、QueueWithSizes                            |

---

## 四、关键 rquickjs 宏与模式速查

```rust
// 1. 定义 JS 类
#[rquickjs::class]
#[derive(rquickjs::JsLifetime)]
pub struct MyClass<'js> {
    field: String,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> MyClass<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>, arg: String) -> Result<Self> { ... }

    #[qjs(get)]
    fn field(&self) -> &str { &self.field }

    #[qjs(set, rename = "field")]
    fn set_field(&mut self, value: String) { self.field = value; }
}

// 2. 注册到全局
Class::<MyClass>::define(&globals)?;

// 3. 异步函数
Func::from(Async(move |ctx, arg| {
    async move { Ok(result) }
}));

// 4. Promise
Promise::wrap_future(&ctx, async move { Ok(value) })

// 5. 持久化 JS 函数
let persistent = Persistent::<Function>::save(ctx, cb);
let restored = persistent.restore(&ctx)?;
restored.call::<_, ()>(())?;

// 6. Ctx userdata（模块级状态）
ctx.store_userdata(MyState::default())?;
let state = ctx.userdata::<MyState>()?;
```

---

_LLRT 源码仅供学习参考，不直接引入依赖。现代运行时的 API 设计以 WinterTC 为标准，实现方式参考 LLRT 的 Rust 层桥接模式。_
