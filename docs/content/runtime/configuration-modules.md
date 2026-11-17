---
title: Configuration Modules
description: 将 JSON、TOML、YAML 等配置文件作为 ESM 默认导出使用。
---

# Configuration Modules

配置文件可以直接导入为数据，而不必在程序中手动读取再解析。

## JSON 示例

创建 `settings.json`：

```json
{
  "name": "hello-vole",
  "port": 3000
}
```

在 `main.js` 中导入：

```js
import settings from "./settings.json";
console.log(settings.name, settings.port);
```

## 其它格式

加载器还支持 JSONC、JSON5、TOML、YAML 和 INI，格式由扩展名识别。模块提供默认导出，不要假设所有键都能作为具名导出。

配置文件支持不等于存在统一的应用配置框架。CLI 不会因为文件名是 `.volerc` 就自动读取它；应用环境变量见 [Environment Variables](/runtime/environment/)。
