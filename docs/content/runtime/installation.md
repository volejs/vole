---
title: Installation
description: 从源码构建 Vole CLI，并将本地二进制加入 PATH。
---

# Installation

当前文档以从源码构建为基准。需要 Rust 工具链，以及构建本机原生依赖所需的编译工具。

## 构建 CLI

```sh
git clone https://github.com/volejs/vole.git
cd vole
cargo build -p vole-cli
./target/debug/vole --version
```

开发构建位于 `target/debug/vole`。需要优化构建时使用：

```sh
cargo build --release -p vole-cli
./target/release/vole --version
```

## 在当前终端使用

在仓库根目录执行：

```sh
export PATH="$PWD/target/debug:$PATH"
vole --help
```

这个 PATH 修改只对当前终端有效。也可以始终使用二进制的完整路径。

## 下一步

按照 [Quickstart](/runtime/quickstart/) 创建程序。出现构建或运行问题时，先记录 `vole version` 的输出，再参考 [Troubleshooting](/runtime/troubleshooting/)。
