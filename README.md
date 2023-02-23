# ZROJ Core

本项目包含了 ZROJ 的所有核心库，**不包含任何可执行文件**。

## How to Contribute

开发环境：

- 请使用 VS Code 作为编辑器，并安装 rust-analyzer 插件。
- 请掌握 Git 的基本用法，（配合 SSH key 验证的使用）。

要新建模块，请使用 `cargo new <模块名称> --lib` 命令，并在本目录下的 Cargo.toml 的 members 中添加上对应的文件夹名字。`--lib` 指创建一个库项目，而不是可执行文件项目。本项目只包含 ZROJ 的核心库，除了 `tools` 之外不应当有任何可执行文件的子项目。为简化版本管理，请继承 workspace 的版本号，简单来说你需要将新建项目的 `Cargo.toml` 内容修改为

```toml
[package]
name = "模块名称，通常不用修改它"
version.workspace = true
edition.workspace = true

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
```

为了降低开发难度，请大家用**中文或者英文**为自己编写的公开的函数、结构体和 traits 提供描述和注意事项。

每个模块 dependencies 应当分开管理，也就是说在对应的模块子目录下调用 `cargo` 命令。项目根目录下的 `Cargo.toml` 应当只用于公共的依赖（这些依赖需要在商讨后再确定）。

同理，每个模块的版本号也当分开管理。

请遵循 [Conventional Commits 规范](https://www.conventionalcommits.org/zh-hans/v1.0.0/) 填写 commit 信息。

工具构建：

```bash
cargo build -p tools --bin cargo-zrdev --release
# 使用 --target x86_64-unknown-linux-gnu 可以进行交叉编译
# 详见 https://doc.rust-lang.org/cargo/commands/cargo-build.html#compilation-options
# 生成的可执行文件位于 target/release 目录下
```

对于 Linux 用户，执行 `cp target/release/cargo-zrdev ~/.cargo/bin` 可以将其安装到本机，它的作用是为 cargo 添加了一个子命令，使用 `cargo zrdev` 可以查看用法（等价于 `cargo-zrdev zrdev`），MacOS 和 Windows 用户待定。你需要保证本机上安装有 Git 命令行工具。

使用 `cargo clean` 来删除构建的内容。