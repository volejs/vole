# Vole Runtime API 规划

> 本文档定义 `vole` JavaScript/TypeScript 运行时的公开 API，包括全局 Web API、内置模块 `vole:` 命名空间以及 CLI 命令接口。

---

## 目录

- [设计原则](#设计原则)
- [全局 Web API](#全局-web-api)
- [内置模块 `vole:`](#内置模块-std)
  - [vole:fs](#stdfs)
  - [vole:path](#stdpath)
  - [vole:env](#stdenv)
  - [vole:process](#stdprocess)
  - [vole:http](#stdhttp)
  - [vole:tcp / vole:udp / vole:socket](#stdtcp--stdudp--stdsocket)
  - [vole:ffi](#stdffi)
  - [vole:test](#stdtest)
- [CLI 命令](#cli-命令)
- [实现路线图](#实现路线图)
- [待讨论议题](#待讨论议题)

---

## 设计原则

1. **Web 标准优先**：首要对齐 TC39 ECMAScript 规范、WinterCG 运行时共识规范以及浏览器 Web API。不追求与 Node.js API 兼容，Node.js 仅作设计参考。
2. **分层实现**：底层 Rust 原生实现 → JavaScript shim 封装 → 用户代码。
3. **命名空间隔离**：系统级能力置于 `vole:` 前缀模块中；Web 标准 API 直接挂载于 `globalThis`。
4. **异步优先**：I/O 操作以 `Promise` 为基础 API，保留必要的 `*Sync` 同步版本用于启动时配置等场景。
5. **类型友好**：API 签名设计时同步考虑 TypeScript 类型声明的可表达性。

---

## rquickjs 生态选型参考

> 本文档的 API 规划基于 rquickjs（QuickJS Rust 绑定）构建。以下是生态调研结果，决定哪些 crate 直接引入、哪些仅作参考。

### 直接引入的 crate

| crate                | 版本    | 用途                                                                       | 引入方式                           |
| -------------------- | ------- | -------------------------------------------------------------------------- | ---------------------------------- |
| `rquickjs`           | `0.12`  | 核心绑定（已引入）                                                         | `features = ["futures", "loader"]` |
| `rquickjs-serde`     | `0.6.1` | Serde 序列化桥接。Rust 值 ↔ JS 值自动转换，`structuredClone`、配置解析必需 | 已引入                             |
| `rquickjs-extension` | —       | 扩展注册框架。依赖 rquickjs 0.9，与项目 0.12 冲突，暂不引入                | 暂不引入                           |

### 仅作参考（不引入依赖）

| 来源                                  | 对应模块                    | 参考价值                                                               |
| ------------------------------------- | --------------------------- | ---------------------------------------------------------------------- |
| **AWS LLRT** `llrt_modules::crypto`   | `crypto` 全局 API           | **极高**。SHA、AES、HMAC、RSA 的 rquickjs 绑定方式                     |
| **AWS LLRT** `llrt_modules::fs`       | `vole:fs`                    | **高**。文件操作 Rust → JS 的异步封装模式                              |
| **AWS LLRT** `llrt_modules::fetch`    | `fetch` / `vole:http`        | **高**。`reqwest` + rquickjs `Response` 对象封装                       |
| **AWS LLRT** `llrt_modules::timers`   | `setTimeout` 等             | **中**。tokio timer + rquickjs callback 调度                           |
| **AWS LLRT** `llrt_modules::encoding` | `TextEncoder`/`TextDecoder` | **中**。字符串 ↔ Uint8Array 转换                                       |
| **AWS LLRT** `llrt_modules::streams`  | `ReadableStream` 等         | **中**。流式数据在 rquickjs 中的表达                                   |
| `rquickjs-extra`                      | `console` / `os` / `timers` | **低**。我们已有 console 基础实现，且目标是 WinterTC 而非 Node.js 兼容 |
| `rquickjs-extra-url`                  | `URL` / `URLSearchParams`   | **低**。直接用 `url` crate 更可控                                      |

### 关键洞察

AWS LLRT 的模块实现方式和我们"Rust 原生实现 → JS shim 封装 → 用户代码"的分层策略**完全一致**——不是做 Node.js JS shim，而是直接在 Rust 层实现完整 API 暴露给 JS。其源码是最有价值的参考。

---

## 全局 Web API

### Phase 0 — 基础运行时

| API                                      | 说明                                           | 实现方式                                                       |
| ---------------------------------------- | ---------------------------------------------- | -------------------------------------------------------------- |
| `console.log(...args)`                   | 标准输出                                       | 已有：`println!` 桥接                                          |
| `console.info(...args)`                  | 标准输出（info 级别）                          | 新增：同 `console.log`                                         |
| `console.warn(...args)`                  | 标准错误（warn 级别）                          | 已有：`eprintln!` 桥接                                         |
| `console.error(...args)`                 | 标准错误（error 级别）                         | 已有：`eprintln!` 桥接                                         |
| `console.debug(...args)`                 | 标准输出（debug 级别）                         | 新增：同 `console.log`                                         |
| `console.time(label?)`                   | 启动计时器                                     | 新增：`HashMap<String, Instant>` 维护                          |
| `console.timeEnd(label?)`                | 结束计时器并输出耗时                           | 新增：同上                                                     |
| `console.assert(condition, ...args)`     | 条件断言失败时输出错误                         | 新增：同 `console.error`                                       |
| `console.table(data, columns?)`          | 以表格形式打印数据（简化版）                   | 新增：格式化输出                                               |
| `setTimeout(callback, delay?, ...args)`  | 延迟执行回调                                   | Rust：`tokio::time::sleep` + `ctx.spawn`                       |
| `clearTimeout(id)`                       | 取消延迟任务                                   | Rust：`AbortHandle` 取消 tokio task                            |
| `setInterval(callback, delay?, ...args)` | 周期性执行回调                                 | Rust：`tokio::time::interval`                                  |
| `clearInterval(id)`                      | 取消周期性任务                                 | Rust：`AbortHandle` 取消 tokio task                            |
| `queueMicrotask(callback)`               | 将回调加入微任务队列                           | Rust：`rquickjs::Promise` + `ctx.spawn`                        |
| `performance.now()`                      | 返回单调高精度时间戳（毫秒，带小数，精度 5μs） | Rust：`std::time::Instant::elapsed()`                          |
| `atob(data)`                             | Base64 解码                                    | Rust：`base64` crate 或标准库                                  |
| `btoa(data)`                             | Base64 编码                                    | Rust：`base64` crate 或标准库                                  |
| `DOMException(message, name?)`           | Web API 标准错误类型                           | JS shim：`Error` 子类封装                                      |
| `reportError(error)`                     | 报告错误到全局                                 | JS shim：打印到 stderr（Phase 0）；Phase 2 后触发 `error` 事件 |

### Phase 1 — 编码与数据

| API                                | 说明                  | 实现方式                                        |
| ---------------------------------- | --------------------- | ----------------------------------------------- |
| `TextEncoder`                      | 字符串 → Uint8Array   | Rust：`String::encode_utf8`                     |
| `TextDecoder`                      | Uint8Array → 字符串   | Rust：`String::from_utf8_lossy`                 |
| `URL`                              | URL 解析与操作        | Rust：`url` crate 桥接                          |
| `URLSearchParams`                  | 查询参数解析与序列化  | Rust：`url::form_urlencoded`                    |
| `URLPattern`                       | URL 模式匹配（路由）  | Rust：`regex` / `matchit` crate                 |
| `Blob`                             | 不可变二进制大对象    | Rust：`Vec<u8>` 包装                            |
| `File`                             | `Blob` 子类，含元信息 | Rust：`Vec<u8>` + 文件名/时间戳                 |
| `structuredClone(value, options?)` | 结构化克隆（深拷贝）  | Rust：`serde_json` 序列化往返（简化）或 V8 算法 |

### Phase 2 — 事件系统

| API               | 说明           | 实现方式                                      |
| ----------------- | -------------- | --------------------------------------------- |
| `Event`           | 事件基类       | JS shim：标准 Event 构造函数                  |
| `EventTarget`     | 事件目标基类   | JS shim：`addEventListener` / `dispatchEvent` |
| `CustomEvent`     | 自定义事件     | JS shim：继承 Event                           |
| `AbortController` | 请求取消控制器 | JS shim：`AbortSignal` + 手动触发 abort       |
| `AbortSignal`     | 取消信号       | Rust：`tokio_util::sync::CancellationToken`   |
| `MessageChannel`  | 消息通道       | JS shim：一对关联的 MessagePort               |
| `MessagePort`     | 消息端口       | JS shim：`EventTarget` + `postMessage`        |
| `MessageEvent`    | 消息事件       | JS shim：继承 Event                           |

### Phase 3 — 网络请求

| API                   | 说明                 | 实现方式                                          |
| --------------------- | -------------------- | ------------------------------------------------- |
| `fetch(input, init?)` | 发起 HTTP/HTTPS 请求 | Rust：`reqwest`（和 `vole:http` 客户端共享连接池） |
| `Request`             | Request 对象         | Rust：`reqwest::Request` 包装                     |
| `Response`            | Response 对象        | Rust：`reqwest::Response` 包装                    |
| `Headers`             | HTTP 头集合          | Rust：`reqwest::header::HeaderMap` 包装           |
| `FormData`            | multipart 表单构造   | Rust：`multipart` crate 或 `reqwest::multipart`   |

### Phase 4 — 加密

| API                                                            | 说明               | 实现方式                                              |
| -------------------------------------------------------------- | ------------------ | ----------------------------------------------------- |
| `crypto`（全局对象）                                           | 加密入口           | Rust：`getrandom` / `rsa` / `sha2` / `aes-gcm` 等包装 |
| `Crypto`（构造函数）                                           | 创建独立 crypto    | JS shim：返回 `crypto` 副本                           |
| `SubtleCrypto`                                                 | 底层加密 API       | Rust：`rsa` / `sha2` / `aes-gcm` / `hmac` 等 crate    |
| `CryptoKey`                                                    | 密钥对象           | Rust：key handle 包装（AES/HMAC/RSA）                 |
| `crypto.getRandomValues(array)`                                | 密码学安全随机数   | Rust：`getrandom` crate                               |
| `crypto.subtle.digest(algorithm, data)`                        | 哈希（SHA-256 等） | Rust：`sha1` / `sha2`                                 |
| `crypto.subtle.encrypt(algorithm, key, data)`                  | 对称加密           | Rust：`aes` / `cbc` / `aes-gcm`                       |
| `crypto.subtle.decrypt(algorithm, key, data)`                  | 对称解密           | Rust：`aes` / `cbc` / `aes-gcm`                       |
| `crypto.subtle.sign(algorithm, key, data)`                     | 数字签名           | Rust：`hmac` / `rsa`                                  |
| `crypto.subtle.verify(algorithm, key, signature, data)`        | 签名验证           | Rust：`hmac` / `rsa`                                  |
| `crypto.subtle.generateKey(algorithm, extractable, keyUsages)` | 密钥生成           | Rust：`getrandom` / `rsa`                             |

### Phase 3 — 流（已完成）

| API                   | 说明            | 实现方式                                      |
| --------------------- | --------------- | --------------------------------------------- |
| `ReadableStream`      | 可读流          | JS shim：async pull + desiredSize 背压        |
| `WritableStream`      | 可写流          | JS shim：write 队列 + ready 背压              |
| `TransformStream`     | 转换流          | JS shim：ReadableStream + WritableStream 组合 |
| `TextEncoderStream`   | 流式 UTF-8 编码 | 基于 TransformStream                          |
| `TextDecoderStream`   | 流式 UTF-8 解码 | 基于 TransformStream                          |
| `CompressionStream`   | 流式压缩        | Rust：`flate2` + TransformStream              |
| `DecompressionStream` | 流式解压缩      | Rust：`flate2` + TransformStream              |

---

## 模块系统

### 核心原则：零幽灵依赖

**外部模块**必须通过 `vole.json` 显式声明。`vole:` 内置模块和 `./` / `../` 相对路径模块无需声明，直接可用。运行时对未声明的外部 import 直接报错。

```json
// vole.json
{
  "name": "my-app",
  "dependencies": {
    "lodash": "npm:lodash@^4.17.21",
    "path": "jsr:@std/path@^1.0.0",
    "utils": "gh:mycompany/utils@v1.2.0"
  },
  "registries": {
    "corp": {
      "url": "https://registry.corp.example.com",
      "auth": "env:REGISTRY_TOKEN"
    }
  }
}
```

```javascript
// app.js
import { readFile } from "vole:fs"; // ✅ 内置模块，直接使用 vole: 前缀
import _ from "lodash"; // ✅ 映射到 npm:lodash（需在 vole.json 中声明）
import { helper } from "utils"; // ✅ 映射到 gh:mycompany/utils（需在 vole.json 中声明）
import { something } from "unregistered"; // ❌ 报错：未在 vole.json 中声明
```

### 注册表体系

| 前缀         | 来源                      | 示例                             | 状态                |
| ------------ | ------------------------- | -------------------------------- | ------------------- |
| `vole:`       | 内置标准库                | `vole:fs`、`vole:tcp`、`vole:udp`   | ✅ 已有             |
| `./` / `../` | 相对路径                  | `./utils.js`                     | ✅ 已有             |
| `npm:`       | npm 注册表                | `npm:lodash@4.17.21`             | 🔵 未来             |
| `jsr:`       | JSR (JavaScript Registry) | `jsr:@std/path@1.0.0`            | 🔵 未来             |
| `gh:`        | GitHub 仓库               | `gh:awslabs/llrt@v0.8.0`         | 🔵 未来             |
| `https://`   | 直接 URL                  | `https://cdn.example.com/lib.js` | 🔵 未来（暂不实现） |
| 自定义       | 私有注册表                | `corp:internal-lib@1.0.0`        | 🔵 未来             |

内置模块 `vole:` 只是预配置的默认注册表，加载机制与外部模块统一。

### `import.meta`

每个 ESM 模块均暴露以下 `import.meta` 属性：

```javascript
// 当前模块的绝对 file:// URL
import.meta.url; // "file:///path/to/module.ts"

// 解析模块标识符为绝对 URL 字符串
import.meta.resolve("./helper.js"); // "file:///path/to/helper.js"

// 是否为入口模块
import.meta.main; // true | false
```

- `import.meta.url` 和 `import.meta.resolve` 的行为对齐浏览器 ESM 规范
- `import.meta.main` 用于判断当前模块是否通过 CLI 直接执行（如 `vole run main.ts`）
- **不提供** `filename` / `dirname` / `path` / `dir` / `glob`（非标准）

### 全局模块中心

```
~/.vole/
├── cache/           # 模块缓存
│   ├── npm/...
│   ├── jsr/...
│   └── gh/...
├── lock/
│   └── vole.lock  # 精确版本锁定
└── registry/
    └── index.json   # 注册表元数据
```

---

## 内置模块 `vole:`

### 实现方式总览

| 模块          | 核心 Rust 依赖                      | 说明                                   |
| ------------- | ----------------------------------- | -------------------------------------- |
| `vole:fs`      | `std::fs` / `tokio::fs`             | 文件系统操作                           |
| `vole:path`    | `std::path`                         | 路径处理，纯计算                       |
| `vole:env`     | `std::env`                          | 环境变量与运行时信息                   |
| `vole:process` | `std::process` / `tokio::process`   | 进程控制、Shell、子进程                |
| `vole:http`    | `reqwest` / `hyper` / `axum`        | HTTP 客户端（reqwest）+ 服务端（axum） |
| `vole:tcp`     | `tokio::net::TcpStream/TcpListener` | TCP connect/listen/Server              |
| `vole:udp`     | `tokio::net::UdpSocket`             | UDP socket                             |
| `vole:socket`  | `tokio::net::UnixStream`            | Unix domain socket（Unix only）        |
| `vole:ffi`     | `libloading` / `dlopen`             | 动态链接库调用                         |
| `vole:test`    | 自定义测试运行器                    | 类似 Deno test，基于 Rust 调度         |

### vole:fs

文件系统操作模块，同时提供同步与异步 API。

```typescript
// --- 同步 API ---

/** 读取文件内容为 Uint8Array */
export function readFileSync(path: string): Uint8Array;

/** 读取文件内容为 UTF-8 字符串 */
export function readTextFileSync(path: string): string;

/** 将数据写入文件（Uint8Array 或 string） */
export function writeFileSync(path: string, data: Uint8Array | string): void;

/** 检查文件或目录是否存在 */
export function existsSync(path: string): boolean;

/** 创建目录 */
export function mkdirSync(path: string, options?: { recursive?: boolean }): void;

/** 读取目录内容 */
export function readdirSync(path: string): string[];

/** 获取文件/目录元信息 */
export function statSync(path: string): {
  size: number;
  isFile: boolean;
  isDirectory: boolean;
  mtime: Date;
  ctime: Date;
};

/** 删除文件或目录 */
export function removeSync(path: string): void;

/** 重命名/移动文件 */
export function renameSync(oldPath: string, newPath: string): void;

/** 复制文件 */
export function copyFileSync(src: string, dest: string): void;

// --- 异步 API ---

export function readFile(path: string): Promise<Uint8Array>;
export function readTextFile(path: string): Promise<string>;
export function writeFile(path: string, data: Uint8Array | string): Promise<void>;
export function mkdir(path: string, options?: { recursive?: boolean }): Promise<void>;
export function readdir(path: string): Promise<string[]>;
export function stat(
  path: string,
): Promise<{ size: number; isFile: boolean; isDirectory: boolean; mtime: Date; ctime: Date }>;
export function remove(path: string): Promise<void>;
export function rename(oldPath: string, newPath: string): Promise<void>;
export function copyFile(src: string, dest: string): Promise<void>;
```

### vole:path

跨平台路径处理模块。

```typescript
/** 拼接路径片段 */
export function join(...parts: string[]): string;

/** 将路径解析为绝对路径 */
export function resolve(...parts: string[]): string;

/** 获取目录名 */
export function dirname(path: string): string;

/** 获取文件名（可移除扩展名） */
export function basename(path: string, ext?: string): string;

/** 获取扩展名 */
export function extname(path: string): string;

/** 判断是否为绝对路径 */
export function isAbsolute(path: string): boolean;

/** 计算相对路径 */
export function relative(from: string, to: string): string;

/** 解析路径为对象 */
export function parse(path: string): {
  root: string;
  dir: string;
  base: string;
  name: string;
  ext: string;
};

/** 将路径对象格式化为字符串 */
export function format(pathObject: {
  dir?: string;
  root?: string;
  base?: string;
  name?: string;
  ext?: string;
}): string;

/** 平台路径分隔符 */
export const sep: string;

/** 平台路径环境变量分隔符 */
export const delimiter: string;
```

### vole:env

环境变量操作模块。仅提供环境变量的读写能力，不涉及进程控制。

```typescript
/** 获取环境变量 */
export function get(name: string): string | undefined;

/** 设置环境变量 */
export function set(name: string, value: string): void;

/** 检查环境变量是否存在 */
export function has(name: string): boolean;

/** 删除环境变量 */
export function remove(name: string): void;

/** 获取所有环境变量 */
export function entries(): Record<string, string>;

/** 环境变量代理对象（等价于以上函数） */
export const env: {
  get(key: string): string | undefined;
  set(key: string, value: string): void;
  has(key: string): boolean;
  remove(key: string): void;
  entries(): Record<string, string>;
};
```

### vole:process

进程控制与生命周期管理。参考 POSIX 进程语义，非 Node.js `process` 对象兼容。

```typescript
/** 退出进程 */
export function exit(code?: number): never;

/** 异常终止进程 */
export function abort(): never;

/** 获取当前工作目录 */
export function cwd(): string;

/** 切换工作目录 */
export function chdir(path: string): void;

/** 获取进程运行时长（秒） */
export function uptime(): number;

/** 获取内存使用信息 */
export function memoryUsage(): {
  rss: number;
  heapTotal: number;
  heapUsed: number;
  external: number;
};

/** 命令行参数 */
export const args: string[];

/** 主程序路径 */
export const argv0: string;

/** 当前进程 ID */
export const pid: number;

/** 父进程 ID */
export const ppid: number;

/** 操作系统平台 */
export const platform: string;

/** CPU 架构 */
export const arch: string;

/** 运行时版本 */
export const version: string;

/** 各组件版本信息 */
export const versions: Record<string, string>;

// --- 标准 I/O（Phase 0 简化版，不依赖 Streams API） ---

export const stdout: {
  /** 同步写入数据到标准输出 */
  write(data: string | Uint8Array): void;
  /** 异步写入数据到标准输出 */
  writeAsync(data: string | Uint8Array): Promise<void>;
  /** 刷新缓冲区 */
  flush(): void;
};

export const stderr: {
  /** 同步写入数据到标准错误 */
  write(data: string | Uint8Array): void;
  /** 异步写入数据到标准错误 */
  writeAsync(data: string | Uint8Array): Promise<void>;
  /** 刷新缓冲区 */
  flush(): void;
};

// --- Shell 命令执行 ---

export interface ExecOptions {
  cwd?: string;
  env?: Record<string, string>;
  timeout?: number;
  capture?: boolean;
}

export interface ExecResult {
  stdout: string;
  stderr: string;
  status: { success: boolean; code: number };
}

/** 异步执行 Shell 命令 */
export function exec(command: string, options?: ExecOptions): Promise<ExecResult>;

/** 同步执行 Shell 命令 */
export function execSync(command: string, options?: ExecOptions): ExecResult;

// --- 子进程管理 ---

export interface SpawnOptions {
  cwd?: string;
  env?: Record<string, string>;
  stdio?: "inherit" | "piped" | "null";
}

export class ChildProcess {
  readonly pid: number;
  kill(signal?: string): boolean;
  wait(): Promise<{ success: boolean; code: number }>;
  output(): Promise<ExecResult>;
  // stdin/stdout/stderr 已补充基础读写 API；待迁移为 ReadableStream/WritableStream
}

/** 创建子进程 */
export function spawn(command: string, args?: string[], options?: SpawnOptions): ChildProcess;

// --- 事件接口（依赖 EventTarget，Phase 2 后实现） ---
export function on(
  event: "exit" | "uncaughtException",
  handler: (...args: unknown[]) => void,
): void;
```

### vole:http

HTTP 服务端与客户端。

```typescript
// --- 服务端 ---

export interface ServerOptions {
  port?: number;
  hostname?: string;
}

export class Server {
  readonly port: number;
  readonly hostname: string;
  /** 启动并开始监听 */
  listen(): Promise<void>;
  /** 优雅关闭服务器 */
  shutdown(): Promise<void>;
}

/** 创建 HTTP 服务器 */
export function serve(
  handler: (req: Request) => Response | Promise<Response>,
  options?: ServerOptions,
): Server;

// --- 客户端 ---

/** 发起 HTTP 请求（fetch 的底层实现，提供更丰富的控制） */
export function request(url: string, options?: RequestInit): Promise<Response>;
```

### vole:tcp / vole:udp / vole:socket

底层网络通信能力，实际拆分为三个模块：`vole:tcp`、`vole:udp`、`vole:socket`（Unix only）。HTTP 应用层能力见 `vole:http`。

```typescript
export interface ListenOptions {
  transport: "tcp" | "udp" | "unix";
  port?: number;
  hostname?: string;
  path?: string;
}

/** 统一监听入口 */
export function listen(options: ListenOptions): TcpListener | UdpSocket | UnixSocket;

// --- TCP ---

export class TcpListener {
  accept(): Promise<TcpStream>;
  close(): Promise<void>;
  readonly localAddr: string;
}

export class TcpStream {
  read(buffer: Uint8Array): Promise<number>;
  write(data: Uint8Array | string): Promise<number>;
  close(): Promise<void>;
  readonly localAddr: string;
  readonly peerAddr: string;
  setNoDelay(noDelay: boolean): void;
  setKeepalive(seconds?: number): void;
}

// --- UDP ---

export class UdpSocket {
  send(data: Uint8Array | string, address: string): Promise<number>;
  recv(buffer: Uint8Array): Promise<{ bytes: number; address: string }>;
  close(): Promise<void>;
  readonly localAddr: string;
}

// --- Unix Domain Socket ---

export class UnixSocket {
  constructor(path: string);
  connect(): Promise<void>;
  read(buffer: Uint8Array): Promise<number>;
  write(data: Uint8Array | string): Promise<number>;
  close(): Promise<void>;
}
```

### vole:ffi

外部函数接口（FFI），基于 `libffi` + `libloading` 调用动态链接库。

```typescript
export interface FunctionDef {
  /** 参数类型 */
  parameters: FfiType[];
  /** 返回类型 */
  result: FfiType;
}

export type FfiType =
  | "void"
  | "bool"
  | "i8"
  | "i16"
  | "i32"
  | "i64"
  | "u8"
  | "u16"
  | "u32"
  | "u64"
  | "f32"
  | "f64"
  | "usize"
  | "isize"
  | "pointer"
  | "string"
  | "cstring";

/** 加载动态链接库，返回句柄 */
export function open(path: string): bigint;

/** 关闭动态链接库句柄 */
export function close(handle: bigint): void;

/** 直接调用符号 */
export function call(handle: bigint, symbol: string, args: unknown[], def: FunctionDef): unknown;

/** 将符号绑定为可复用函数 */
export function define(
  handle: bigint,
  symbol: string,
  def: FunctionDef,
): (...args: unknown[]) => unknown;
```

示例：

```js
import { open, close, call, define } from "vole:ffi";

const lib = open("./libexample.dylib");

const add = define(lib, "add", {
  parameters: ["i32", "i32"],
  result: "i32",
});
console.log(add(1, 2)); // 3

const len = call(lib, "strlen", ["hello"], {
  parameters: ["string"],
  result: "u32",
});
console.log(len); // 5

close(lib);
```

### vole:test

内置测试框架。参考 Web Platform Tests 与 Deno Test 的设计，零配置即可运行。

```typescript
export interface TestDefinition {
  name: string;
  fn: () => void | Promise<void>;
  ignore?: boolean;
  only?: boolean;
  timeout?: number;
}

/** 注册一个测试用例 */
export function test(name: string, fn: () => void | Promise<void>): void;
export function test(def: TestDefinition): void;

/** 分组描述（BDD 风格） */
export function describe(name: string, fn: () => void): void;

/** 单个测试（BDD 风格） */
export function it(name: string, fn: () => void | Promise<void>): void;

/** 全组前置钩子 */
export function beforeAll(fn: () => void | Promise<void>): void;

/** 全组后置钩子 */
export function afterAll(fn: () => void | Promise<void>): void;

/** 每个用例前置钩子 */
export function beforeEach(fn: () => void | Promise<void>): void;

/** 每个用例后置钩子 */
export function afterEach(fn: () => void | Promise<void>): void;
```

**CLI 使用**：

```bash
vole test                  # 运行当前目录及子目录所有 *_test.js / *_test.ts
vole test --watch          # 监听模式
vole test ./foo_test.js    # 运行指定文件
```

## CLI 命令

```
vole version                          # 显示运行时版本号
vole help [command]                   # 显示帮助信息（可指定具体命令）
vole run <file>                       # 执行指定 JS/TS 文件
vole run --watch <file>               # 监听模式执行（文件变更自动重载）
vole eval "<code>"                    # 执行代码字符串
vole repl                             # 启动交互式 REPL
vole test                             # 运行测试（*_test.js / *_test.ts）
vole test --watch                     # 监听模式运行测试
vole <file>                           # vole index.js 的快捷形式，等价于 vole run <file>
```

### CLI 参数约定

| 参数      | 说明                                  |
| --------- | ------------------------------------- |
| `--watch` | 启用文件监听模式（仅限 `run` 子命令） |

---

## 实现路线图

| 阶段        | 全局 Web API                                                                                         | 内置模块 `vole:`                                  | 优先级  |
| ----------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------ | :-----: |
| **Phase 0** | `console` 补全；`performance.now()`；Timers；`atob`/`btoa`；`DOMException`；`reportError`            | `vole:env`；`vole:process`                         |  🔴 高  |
| **Phase 1** | `TextEncoder`/`TextDecoder`；`URL`/`URLSearchParams`/`URLPattern`；`Blob`/`File`；`structuredClone`  | `vole:fs` 异步扩展；`vole:path` 扩展               |  🟡 中  |
| **Phase 2** | `Event`/`EventTarget`/`CustomEvent`；`AbortController`/`AbortSignal`；`MessageChannel`/`MessagePort` | —                                                |  🟡 中  |
| **Phase 3** | `fetch`/`Request`/`Response`/`Headers`/`FormData`                                                    | `vole:http` 服务端/客户端                         |  🟡 中  |
| **Phase 4** | `crypto` / `Crypto` / `SubtleCrypto` / `CryptoKey` 全套（含 RSA pkcs8/spki）                         | `vole:tcp` / `vole:udp` / `vole:socket`；`vole:test` |  🟢 低  |
| **Phase 5** | —                                                                                                    | `vole:ffi`                                        | 🔵 未来 |

---

## 待讨论议题

1. **HTTP 实现策略** ✅ 已决策
   - `fetch` 基于 `reqwest` 实现
   - `vole:http` 客户端基于 `reqwest`，与 `fetch` 共享全局 `reqwest::Client` 连接池
   - `vole:http` 服务端基于 `axum`，可接受 `vole:tcp::listen()` 作为入口（共享 socket endpoint）
   - `vole:tcp` / `vole:udp` / `vole:socket` 独立基于 `tokio::net`，与 `vole:http` 不共享连接池

2. **模块加载策略** ✅ 已决策
   - `vole.json` 显式声明所有依赖，零幽灵依赖
   - 注册表体系：`vole:`（内置）、`npm:`、`jsr:`、`gh:`、自定义私有注册表
   - URL import（`https://`）暂不实现，作为未来特性
   - 不引入独立包管理器 CLI，依赖声明即包管理

3. **权限模型** ✅ 已决策
   - **完全开放**，不内置权限控制
   - 移除 `vole:permissions` 模块
   - 不引入 `--allow` / `--deny` CLI 参数
   - 信任开发者，类似 Node.js / Bun 策略

4. **TypeScript 支持** ✅ 已决策
   - 内置 transpile（参考 Bun，使用 `swc` Rust crate），不做类型检查
   - 支持直接运行 `.ts` 文件：`vole run index.ts`
   - **不支持 `.tsx`/`.jsx`**（仅支持 `.ts`/`.js`）；TSX/JSX 作为未来特性（Phase 6+）
   - **不支持 CommonJS**（`require`/`module.exports`）；仅支持 ESM（`import`/`export`）
   - 读取 `tsconfig.json` 的 `paths` 映射（重要）
   - `--watch` 模式支持 `.ts` 文件监听和自动 transpile
   - 类型检查交给 IDE 和外部 `tsc --noEmit`
   - 内置模块随运行时发布 `.d.ts` 类型声明

5. **错误处理风格** ✅ 已决策
   - JS 层统一 `throw` 标准 `Error` 对象，带 `.code` 属性（如 `ENOENT`、`EACCES`）
   - Rust 实现层内部使用 `Result<T, rquickjs::Error>`
   - 不需要独立的 `vole:errors` 模块

6. **`import.meta` 属性** ✅ 已决策
   - `import.meta.url` — 当前模块的绝对 `file:///` URL
   - `import.meta.resolve(specifier)` — 解析模块标识符为绝对 `file:///` URL 字符串
   - `import.meta.main` — `boolean`，标识当前模块是否为入口模块
   - **不提供** `filename`/`dirname`/`path`/`dir`（可通过 `new URL(import.meta.url).pathname` 派生）
   - **不提供** `import.meta.glob`（Vite 私有扩展，非标准）

---

_本文档随项目迭代持续更新。如有 API 变更，需同步更新此文档及对应实现。_
