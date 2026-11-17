---
title: Encode and decode UTF-8
description: 使用 TextEncoder 和 TextDecoder 在文本与字节之间转换。
category: encoding
---

# Encode and decode UTF-8

```js
const bytes = new TextEncoder().encode("你好，Vole");
console.log(bytes.length);
console.log(new TextDecoder().decode(bytes));
```

保存为 `encoding.js`，运行 `vole run encoding.js`。

UTF-8 字节数不等于 JavaScript 字符串长度。处理二进制数据时应保留字节类型，不要将任意 Unicode 文本直接当作 `btoa` 的输入。相关接口见 [Globals & Web APIs](/modules/globals/)。
