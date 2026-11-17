---
title: Output & Exit Codes
description: 用控制台结果和进程退出状态判断测试成功与失败。
---

# Output & Exit Codes

运行器输出每个用例的结果，并在结束后汇总：

```text
  ✓ adds
1 passed, 0 failed
```

抛出异常或返回 rejected Promise 的测试会记录失败。存在失败用例时，CLI 以非零状态退出；加载和执行期间的错误也应作为失败处理。

## 在自动化中使用

```sh
vole test tests/
```

让 CI 直接检查命令退出状态，不要仅通过匹配控制台中的某一行判断成功。

## 当前不提供

没有内置 JSON、JUnit、覆盖率或自定义 reporter 协议。也没有 snapshot 和 mock 框架，不应把控制台汇总称为完整的报告系统。

测试函数和 hook 的执行边界见 [Test Lifecycle](/test/lifecycle/)。
