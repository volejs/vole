---
title: Read environment variables
description: 通过 vole:env 管理当前进程的环境变量。
category: system
---

# Read environment variables

```js
import { get, set, remove } from "vole:env";

set("VOLE_EXAMPLE", "ready");
console.log(get("VOLE_EXAMPLE"));
remove("VOLE_EXAMPLE");
console.log(get("VOLE_EXAMPLE") ?? "not set");
```

保存为 `environment.js`，运行 `vole run environment.js`。

修改只影响当前进程及其之后创建的子进程，不会改变启动 Vole 的父进程。指定 `.env` 文件的加载方式见 [Environment Variables](/runtime/environment/)。
