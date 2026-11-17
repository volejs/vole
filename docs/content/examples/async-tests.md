---
title: Write an asynchronous test
description: 注册异步测试，用异常表示失败，并交给 vole test 执行。
category: testing
---

# Write an asynchronous test

创建 `async_test.js`：

```js
import { test } from "vole:test";

test("awaits a result", async () => {
  const value = await Promise.resolve(42);
  if (value !== 42) throw new Error("expected 42");
});
```

```sh
vole test async_test.js
```

文件名应以 `_test.js` 或 `_test.ts` 结尾。CLI 会运行注册的用例，不需要在文件末尾手动调用 `run()`。

没有内置断言库、mock 或 snapshot。分组和 hook 见 [vole:test API](/test/api/)。
