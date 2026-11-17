---
title: Compatibility & Limits
description: 已有能力不等于浏览器、Node.js 或完整 npm 兼容。
---

# Compatibility & Limits

Vole 是 ECMAScript 运行时，不是浏览器。对于 DOM、HTML 和 CSS 专属概念，不以复制浏览器环境为目标。

## 当前边界

| 领域 | 当前说明 |
| --- | --- |
| TypeScript | 转译执行，不内置类型检查。 |
| 模块 | 显式 ESM 路径和 import map，不是完整 Node.js 解析。 |
| npm | 可以下载直接依赖，不保证包在运行时兼容。 |
| 调试 | 没有内置 inspector 或断点调试命令。 |
| Workers | 尚无公开实现。 |
| WebAssembly | 部分实现，不能视为完整 WebAssembly JS API。 |
| TLS 模块 | `vole:tls` 仍是不可用的占位接口。 |
| Unix socket | 存在平台限制。 |

## Web API 取舍

支持 URL、fetch 或事件接口，不意味着提供 `document`、DOM 节点、浏览器布局或浏览器存储模型。需要系统能力时使用专用 `vole:*` 模块。

## 查询详细状态

以仓库 [ROADMAP.md](https://github.com/volejs/vole/blob/main/ROADMAP.md) 为详细状态入口。遇到文档和行为不一致时，请附版本信息和最小复现报告，不要依赖未列明的兼容行为。
