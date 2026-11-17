---
title: Quickstart
description: 创建并执行 JavaScript 程序，然后运行一个最小测试。
---

# Quickstart

先完成 [安装](/runtime/installation/)，确保终端可以找到 `vole`。

## 创建项目

```sh
mkdir hello-vole
cd hello-vole
vole init hello-vole .
```

初始化会写入 `package.json` 和默认 `main.js`。请在新目录中操作，避免覆盖已有的 `package.json`。

将 `main.js` 改为：

```js
const message = await Promise.resolve("Hello, Vole!");
console.log(message);
```

## 执行程序

```sh
vole main.js
vole run main.js
vole eval 'console.log(1 + 2)'
```

前两个命令都执行文件，第三个直接执行传入的代码。

## 写一个测试

创建 `hello_test.js`：

```js
import { test } from "vole:test";

test("adds numbers", () => {
  if (1 + 2 !== 3) throw new Error("unexpected sum");
});
```

```sh
vole test hello_test.js
```

测试运行器没有内置断言库；用异常表示失败。继续阅读 [TypeScript](/runtime/typescript/)、[模块解析](/runtime/module-resolution/) 和 [测试执行](/test/execution/)。
