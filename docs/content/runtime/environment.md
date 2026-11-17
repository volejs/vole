---
title: Environment Variables
description: 使用 vole:env 读取、修改环境变量并加载指定配置。
---

# Environment Variables

```js
import { get, set, has, remove } from "vole:env";

set("VOLE_MODE", "development");
console.log(get("VOLE_MODE"));
console.log(has("VOLE_MODE"));
remove("VOLE_MODE");
```

## 加载文件

```js
import { load, get } from "vole:env";

load("./config/.env");
console.log(get("VOLE_MODE"));
```

`load(path)` 必须显式传入路径。CLI 启动时还会尝试加载当前工作目录的 `.env`，不要把两种行为混淆。

## 使用边界

环境变量修改影响当前进程及之后创建的子进程，不会修改父进程。不要将密钥写入示例源码或提交到版本库。

完整 API、变量名限制与实现位置见 [vole:env](/modules/env/)。
