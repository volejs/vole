---
title: File Types
description: 区分脚本入口、TypeScript 转译和配置模块。
---

# File Types

## 脚本

JavaScript 入口以 `.js` 为基本用法。TypeScript 入口支持 `.ts`、`.mts` 和 `.tsx` 的转译路径；这不代表完整的 JSX 框架或构建工具兼容。

模块加载器还识别 `.cts` 的 TypeScript 转译，但不要据此推断 CommonJS 支持，或将模块加载能力等同于 CLI 入口能力。

## 配置模块

模块加载器可处理以下配置文件：

| 类型 | 扩展名 |
| --- | --- |
| JSON | `.json` |
| JSON with comments | `.jsonc` |
| JSON5 | `.json5` |
| TOML | `.toml` |
| YAML | `.yaml`、`.yml` |
| INI | `.ini` |

配置模块提供默认导出；示例见 [Configuration Modules](/runtime/configuration-modules/)。

## 显式引用

导入文件时写出路径和扩展名：

```js
import { greet } from "./greet.js";
```

不要依赖扩展名补全、目录入口或 Node.js 的包解析约定。
