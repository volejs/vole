---
title: REPL
description: 在终端中交互执行简短的 ECMAScript 表达式。
---

# REPL

```sh
vole repl
```

REPL 适合计算表达式、检查语言行为和尝试简短代码：

```js
1 + 2
console.log("hello")
typeof URL
```

输入 `exit` 或 `quit` 退出。

## 使用边界

当前 REPL 按行读取输入，不是完整 IDE 或调试器。多行程序、文件模块和异步生命周期示例建议写入文件，然后使用 `vole run`。

REPL 不意味着支持浏览器 DOM、Node.js 全局对象或 inspector 调试协议。文件执行的排错方式见 [Troubleshooting](/runtime/troubleshooting/)。
