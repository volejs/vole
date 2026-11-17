---
title: Make an HTTP request
description: 使用 fetch 读取响应，并区分网络异常和 HTTP 错误状态。
category: network
---

# Make an HTTP request

保存为 `request.js`，在可以访问互联网的环境运行：

```js
const response = await fetch("https://example.com/");
if (!response.ok) {
  throw new Error(`HTTP ${response.status}`);
}
console.log(await response.text());
```

```sh
vole run request.js
```

网络错误会使请求失败；HTTP 404 等状态仍然产生 Response，需要显式检查 `ok`。更多接口见 [Network overview](/runtime/networking/)。
