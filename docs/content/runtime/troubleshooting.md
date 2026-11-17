---
title: Troubleshooting
description: 从版本、模块路径和异步错误入手定位运行问题。
---

# Troubleshooting

## 先缩小问题

```sh
vole version
vole --help
vole run main.js
```

保留报错、入口代码、操作系统和版本信息。先去掉 Watch Mode，用单次执行确认问题是否仍然存在。

## 常见原因

| 现象 | 排查方向 |
| --- | --- |
| 找不到模块 | 检查显式扩展名、相对路径和当前目录的 import map。 |
| npm 包无法运行 | 下载成功不代表 Node.js API、CommonJS 或包解析兼容。 |
| TypeScript 没有报告类型错误 | 当前只转译，不提供类型检查。 |
| 修改文件没有重跑 | Watch Mode 不递归追踪完整依赖图。 |
| 异步任务没有按预期完成 | 检查 Promise 是否被等待，以及 managed task 的生命周期。 |

## 调试边界

当前没有内置 inspector 或断点调试命令。可以使用 `console.log`、捕获异常和最小复现代码逐步定位；不要把浏览器开发者工具或 Node.js 调试器作为默认能力。

进一步阅读 [模块解析](/runtime/module-resolution/)、[异步生命周期](/runtime/lifecycle/) 和 [兼容性](/runtime/compatibility/)。
