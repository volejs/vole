---
title: Test execution
description: 发现测试文件、注册测试并通过 CLI 执行。
---

# Test execution

## 文件发现

```sh
vole test
vole test tests/
vole test tests/example_test.js
```

目录扫描递归发现 `*_test.js` 和 `*_test.ts`，跳过隐藏目录及 `target`、`node_modules`、`.git`。

## 最小测试

创建 `tests/math_test.js`：

```js
import { test } from "vole:test";

test("adds numbers", async () => {
  const value = await Promise.resolve(1 + 2);
  if (value !== 3) throw new Error("expected 3");
});
```

测试文件负责注册；`vole test` 会统一调用运行器，不需要在每个文件中手动 `run()`。

## 隔离边界

当前 CLI 在一次运行中使用同一个 VM，依次加载测试文件；不能将文件边界视为进程或全局状态隔离。用例应避免污染共享状态，并显式清理资源。

没有并行测试、watch 测试、`test.only` 或 `test.skip` 的兼容承诺。注册与 hook 见 [API](/test/api/)，执行结果见 [Output & Exit Codes](/test/output/)。
