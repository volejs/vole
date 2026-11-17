---
title: Explicit Imports & Import Maps
description: 使用显式 ESM 路径和 importmap.json，避免隐式裸依赖。
---

# Explicit Imports & Import Maps

优先使用明确的本地模块路径：

```js
import { greet } from "./src/greet.js";
```

## 定义别名

在启动程序的当前工作目录创建 `importmap.json`：

```json
{
  "imports": {
    "greeting": "./src/greet.js",
    "app/": "./src/"
  }
}
```

```js
import { greet } from "greeting";
import { value } from "app/value.js";
```

解析支持精确映射和前缀映射；多个前缀匹配时选择最长的前缀。请从放置该配置的项目目录启动程序，并为映射值使用明确的本地文件路径。

## 不意味着完整包管理

import map 不是 npm 的自动解析器。当前不承诺远程 URL 模块、完整 import map 标准或 package exports 兼容。

拒绝未知裸标识符可以减少隐式依赖，但当前没有完整的包依赖图校验。因此这里不将其描述成已经完成的“直接依赖强制检查”功能。
