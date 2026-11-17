---
title: Async lifecycle
description: 理解 Promise、定时器和 managed task 的收尾边界。
---

# Async lifecycle

Vole 的 VM 基于异步 QuickJS 上下文和 Tokio。入口文件可以等待异步结果：

```js
const result = await Promise.resolve("ready");
console.log(result);
```

## 等待你发起的工作

优先直接 `await` 需要完成的 Promise，或者把 Promise 返回给调用方。不要将任意未跟踪的后台 Promise 当作保活保证。

CLI 会处理定时器和受管理的 `vole:until` 任务收尾，但存在超时边界，不是无限等待一切异步活动。

## 后台任务

需要显式管理任务生命周期时，参考 [vole:until](/modules/until/)。运行时嵌入方式见 [Vole Runtime](/runtime/vm/)。

## 测试生命周期

测试函数返回的 Promise 会被等待。套件 hook 与测试执行顺序见 [Test Lifecycle](/test/lifecycle/)。

## 并发边界

异步 I/O 不等同于 Worker 线程。当前没有公开的 Workers API，也没有可作为并发 API 使用的 ModuleGraph。
