---
title: Import a local module
description: 使用显式路径拆分 ESM 文件，不依赖隐式裸包解析。
category: modules-and-packages
---

# Import a local module

在同一目录创建 `greet.js`：

```js
export function greet(name) {
  return `Hello, ${name}!`;
}
```

创建 `main.js`：

```js
import { greet } from "./greet.js";
console.log(greet("Vole"));
```

```sh
vole run main.js
```

保留相对路径和扩展名。需要别名时使用 [import map](/runtime/import-maps/)；[安装 npm 包](/pm/install/) 不会自动启用完整 Node.js 模块解析。
