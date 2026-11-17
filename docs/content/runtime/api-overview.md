---
title: API Overview
description: 区分 ECMAScript、标准 Web API、公开系统模块和内部绑定。
---

# API Overview

## ECMAScript

语言执行基于 QuickJS。JavaScript 对象、Promise 和 ESM 构成应用的基础。

## 标准 Web API

适用于非浏览器环境的接口以全局对象提供，包括 URL、编码、事件、fetch、Streams 和 Web Crypto 等。接口清单与具体限制见 [Globals & Web APIs](/modules/globals/)。

这些接口参考 WinterTC，但不表示实现完整浏览器平台或已经通过所有兼容性测试。

## 系统能力

文件、进程、环境变量和底层网络通过显式导入的 `vole:*` 模块提供：

```js
import { readTextFile } from "vole:fs";
import { join } from "vole:path";

console.log(await readTextFile(join("data", "message.txt")));
```

示例需要预先创建 `data/message.txt`。公开模块入口见 [Modules](/modules/overview/)。

## 内部绑定

`vole:raw:*` 是 JS shim 使用的底层桥接层，不作为应用的稳定入口。扩展机制和原生模块架构放在 Runtime 的 Internals 分组，而不是与应用 API 混在一起。
