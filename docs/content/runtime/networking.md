---
title: Network overview
description: 按 HTTP、TCP、UDP 和 DNS 任务选择现有网络接口。
---

# Network overview

## HTTP 请求

标准 HTTP 请求优先使用全局 `fetch`：

```js
const response = await fetch("https://example.com/");
if (!response.ok) throw new Error(`HTTP ${response.status}`);
console.log(await response.text());
```

该示例需要网络访问。传输失败和非成功状态需要分别处理；`fetch` 不会仅因为 HTTP 404 就抛错。

## 选择模块

| 需求 | 接口 |
| --- | --- |
| 标准请求与响应对象 | [Globals & Web APIs](/modules/globals/) |
| HTTP 客户端与服务端 | [vole:http](/modules/http/) |
| TCP 连接与监听 | [vole:tcp](/modules/tcp/) |
| UDP socket | [vole:udp](/modules/udp/) |
| DNS 查询 | [vole:dns](/modules/dns/) |
| Unix domain socket | [vole:socket](/modules/socket/) |

这些是不同层次的接口，不要将原始连接直接视为 HTTP 服务。

## 限制

`vole:tls` 当前是不可用的能力占位，见 [模块说明](/modules/tls/)。这与底层 HTTP 客户端处理 HTTPS 是两回事。Unix domain socket 有平台限制；没有在这里承诺 WebSocket 服务端能力。
