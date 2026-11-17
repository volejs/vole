---
title: Module Resolution
description: 通过显式文件路径、内置模块和 import map 解析模块。
---

# Module Resolution

Vole 使用 ESM 模块。常见导入分为三类：

```js
import { readTextFile } from "vole:fs";
import { greet } from "./greet.js";
import settings from "./settings.json";
```

## 内置模块

`vole:*` 是运行时注册的模块命名空间。公开模块见 [Modules](/modules/overview/)；应用不应依赖 `vole:raw:*` 内部接口。

## 文件模块

相对导入基于导入方文件定位。请使用显式路径和扩展名，不要假设支持 Node.js 的目录或包入口推断。

## 裸标识符

未被 import map 映射的裸标识符会被拒绝：

```js
// 不会因为 node_modules 中存在同名目录而自动成功
import library from "some-library";
```

需要短名称时使用 [Explicit Imports & Import Maps](/runtime/import-maps/)。当前实现不是完整的 package exports 或依赖声明校验器。

## 获取依赖与执行依赖

`vole install` 负责下载包，模块解析负责定位文件。这是不同环节，不能把下载成功当作包兼容性的保证。
