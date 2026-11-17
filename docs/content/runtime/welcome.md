---
title: Welcome to Vole
description: 从小型 ECMAScript 运行时开始，了解 Vole 的能力和边界。
---

# Welcome to Vole

Vole 是基于 QuickJS 和 Rust 的 ECMAScript 运行时，不是浏览器，也不是 Node.js 兼容层。它将标准 Web API 与显式导入的系统模块组合在一起。

## 开始使用

1. 按照 [Installation](/runtime/installation/) 从源码构建。
2. 通过 [Quickstart](/runtime/quickstart/) 运行第一个程序。
3. 在 [Examples](/examples/) 中找到面向具体任务的例子。

## 选择入口

- [Runtime](/runtime/vm/)：VM、模块加载与异步生命周期。
- [Package Manager](/pm/overview/)：初始化项目和获取 npm 包。
- [Modules](/modules/overview/)：标准 Web API 与 `vole:*` 模块参考。
- [Test](/test/overview/)：测试发现、注册和运行。

## 设计原则

优先使用 ECMAScript 和适用于非浏览器环境的 Web API。文件系统、进程、网络连接和 FFI 通过专用模块暴露，不依赖隐式 Node.js 全局对象。

TypeScript 执行会移除类型并转译，不进行类型检查。包能够被下载不等于它可以在 Vole 中运行；请先阅读 [兼容性与限制](/runtime/compatibility/)。
