---
title: Project setup
description: 使用 vole init 创建显式的 ESM 项目。
---

# Project setup

`vole init` 创建最小项目，不生成框架、构建流水线或 TypeScript 类型检查配置。

## 新建目录

```sh
vole init my-app ./my-app
cd my-app
vole run main.js
```

第一个参数是项目名，第二个参数才是目标目录。省略目录时默认写入当前目录。

## 生成内容

- `package.json`：项目名、版本、入口、ESM 类型和依赖对象。
- `main.js`：默认的问候程序；已有文件不会覆盖。

已有的 `package.json` 会被重新写入。请不要在未备份的项目中直接初始化。

## 继续组织项目

使用 `./src/example.js` 等显式路径拆分模块。需要短别名时配置 [importmap.json](/runtime/import-maps/)，不要假设裸包名可以自动解析。

完整参数见 [vole init](/pm/init/)，依赖获取见 [vole install](/pm/install/)。
