---
title: TypeScript
description: Vole 转译并执行 TypeScript，但不内置类型检查器。
---

# TypeScript

Vole 使用 Oxc 转译 TypeScript，再交由 QuickJS 执行 JavaScript。

## 运行 TypeScript 文件

创建 `main.ts`：

```ts
const message: string = "Hello, TypeScript!";
console.log(message);
```

```sh
vole run main.ts
```

## 转译不等于检查

类型标注不会在运行时验证值，也不会生成类型检查报告。Vole 没有 `vole check` 命令。

项目需要类型检查时，可独立配置外部 TypeScript 工具链。这里不承诺为所有 Vole API 提供完整的 TypeScript 类型声明。

## 模块边界

TypeScript 同样使用 [显式路径或 import map](/runtime/import-maps/)。文件转译并不会启用 Node.js 包解析、CommonJS 兼容或完整的 npm 运行环境。

支持的文件类型和入口限制见 [File Types](/runtime/file-types/)。
