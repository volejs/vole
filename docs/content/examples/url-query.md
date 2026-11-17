---
title: Build a URL with query parameters
description: 使用全局 URL 和 URLSearchParams，无需导入系统模块。
category: web-apis
---

# Build a URL with query parameters

```js
const url = new URL("/search", "https://example.com/");
url.searchParams.set("q", "hello vole");
url.searchParams.set("page", "1");
console.log(url.href);
console.log(url.searchParams.get("q"));
```

保存为 `url.js`，运行 `vole run url.js`。这个例子只构造 URL，不发起网络请求。

Vole 提供适合非浏览器环境的 Web API，但不提供 DOM。接口概览见 [Globals & Web APIs](/modules/globals/)。
