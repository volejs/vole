---
title: Read a text file
description: 写入 UTF-8 文本，再通过 vole:fs 异步读取。
category: file-system
---

# Read a text file

在可写的目录创建并执行 `files.js`：

```js
import { readTextFile, writeFile } from "vole:fs";

await writeFile("message.txt", "Hello from a file");
console.log(await readTextFile("message.txt"));
```

```sh
vole run files.js
```

示例会创建或覆盖当前目录的 `message.txt`。文件不存在或权限不足时应处理异常。句柄和目录操作见 [vole:fs](/modules/fs/)。
