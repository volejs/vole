---
title: Work with system paths
description: 使用 vole:path 拼接路径并提取文件名。
category: built-in-modules
---

# Work with system paths

```js
import { join, basename, extname } from "vole:path";

const path = join("data", "notes", "hello.txt");
console.log(path);
console.log(basename(path));
console.log(extname(path));
```

保存为 `paths.js`，运行 `vole run paths.js`。拼接字符串不会创建目录或文件。

系统路径受平台规则影响，URL 应使用全局 `URL`。完整函数列表见 [vole:path](/modules/path/)。
