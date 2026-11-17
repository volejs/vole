---
title: Configuration & Limits
description: package.json 的当前用途，以及尚未实现的包管理基础设施。
---

# Configuration & Limits

包管理器目前面向项目初始化和直接依赖下载，不是完整 npm 客户端。

## package.json

`vole init` 生成项目名称、版本、入口、ESM 类型和 `dependencies`。不带包名的 `vole install` 读取当前目录的依赖；带包名的调用安装指定包并写回依赖声明。

## 尚未提供

- 递归解析和安装完整依赖图。
- lockfile 与可重复安装保证。
- 全局缓存、虚拟存储和隔离安装布局。
- bin 链接、workspace 和 peer dependency 解析。
- 自动执行 lifecycle scripts。
- 自动读取 `.volerc` 的配置体系。

scoped package 的参数解析仍有限制，详见 [vole install](/pm/install/)。

## 命令边界

当前公开命令是 [vole init](/pm/init/) 和 [vole install](/pm/install/)。没有 `vole add`、`remove`、`update`、`dedupe` 或 `prune` 命令。

## 与模块解析的关系

`node_modules` 中存在文件不代表裸包名会被接受。使用依赖前还要满足 [模块解析规则](/runtime/module-resolution/)，并确认包没有依赖不支持的 Node.js 或浏览器接口。
