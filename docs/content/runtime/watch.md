---
title: Watch Mode
description: 监听入口目录的文件变化并重新创建 VM 执行程序。
---

# Watch Mode

```sh
vole --watch main.js
```

也可以使用 TypeScript 入口：

```sh
vole --watch main.ts
```

## 当前行为

CLI 监听入口文件所在目录的变化，重新创建 VM 并执行入口。它不是模块热替换：上一次 VM 的内存状态不会保留。

## 限制

- 当前目录监听不是递归的依赖图监听。
- 不保证捕获子目录和目录外依赖的变化。
- 不提供前端 HMR、状态迁移或持久服务协调。
- 执行错误可能结束命令，而不是始终保持监听。

更改未触发重跑时先手动执行入口，检查文件是否位于监听范围内。需要定位错误可参考 [Troubleshooting](/runtime/troubleshooting/)。
