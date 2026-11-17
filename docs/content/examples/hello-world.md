---
title: Hello, Vole
description: 输出消息并等待一个 Promise，运行最小 JavaScript 文件。
category: basics
---

# Hello, Vole

将下面的代码保存为 `main.js`：

```js
const message = await Promise.resolve("Hello, Vole!");
console.log(message);
```

```sh
vole run main.js
```

入口可以直接使用顶层 `await`。也可以使用 `vole main.js` 执行同一个文件。环境准备见 [Quickstart](/runtime/quickstart/)。
