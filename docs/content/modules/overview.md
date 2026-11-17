---
title: Module overview
description: 按任务查找公开的 vole:* 模块和标准 Web API。
---

# Module overview

系统能力通过 `vole:*` 显式导入，标准 Web API 则通过全局对象提供。以下链接指向现有 API 参考。

## 文件、环境与进程

- [vole:fs](/modules/fs/)：文件、目录和文件句柄。
- [vole:path](/modules/path/)：系统路径处理。
- [vole:env](/modules/env/)：环境变量和指定 `.env` 文件。
- [vole:os](/modules/os/)：进程信息、标准流和进程控制。
- [vole:exec](/modules/exec/)：命令执行和子进程。
- [vole:signal](/modules/signal/)：进程信号。

## 网络

- [vole:http](/modules/http/)：HTTP 客户端和服务端。
- [vole:tcp](/modules/tcp/)、[vole:udp](/modules/udp/)、[vole:dns](/modules/dns/)。
- [vole:socket](/modules/socket/)：Unix domain socket，限支持的平台。
- [vole:tls](/modules/tls/)：不可用的占位接口，不是已经实现的 TLS API。

## 其它能力

- [vole:ffi](/modules/ffi/)：动态库调用。
- [vole:until](/modules/until/)：受管理的异步任务。
- [vole:test](/modules/test/)：测试注册与运行。
- [vole:wasm](/modules/wasm/)：WebAssembly 辅助接口，部分实现。
- [Globals & Web APIs](/modules/globals/)：URL、编码、事件、fetch、Streams 和 Crypto 等。

## 公开与内部接口

应用使用公开 `vole:*` 模块。底层 `vole:raw:*` 绑定是实现细节；相关架构见 [Native module architecture](/runtime/modules/)。
