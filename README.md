# ZROJ Core

本项目包含了 ZROJ 的所有核心库，除了开发工具以外，**不包含任何可执行文件**。

## How to Contribute

### 开发环境

- 请使用 VS Code 作为编辑器，并安装 rust-analyzer 插件。
- 请掌握 Git 的基本用法，（配合 SSH key 验证的使用）。

为了保持规范的提交信息和版本管理，我们提供了相应的命令行开发工具 zrdev，构建命令为：

```bash
cargo build -p tools --bin cargo-zrdev --release
# 使用 --target x86_64-unknown-linux-gnu 可以进行交叉编译
# 详见 https://doc.rust-lang.org/cargo/commands/cargo-build.html#compilation-options
# 生成的可执行文件位于 target/release 目录下
```

- 对于 Linux/MacOS 用户，执行 `cp target/release/cargo-zrdev ~/.cargo/bin` 可以将其安装到本机
- 对于 Windows 用户，请将 `target/release/cargo-zrdev.exe` 手动复制到 `$HOME/.cargo/bin` 目录下（即 `C:/User/<你的用户名>/.cargo/bin`）。

安装完成后使用 `cargo --list` 命令检查，你可以发现在列表的末尾多了一个 `zrdev`。

它的作用是为 cargo 添加了一个子命令，使用 `cargo zrdev` 可以查看用法（等价于 `cargo-zrdev zrdev`）。使用本工具的前提是你需要保证本机上安装有 Git 命令行工具。

### 开发流程

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

请遵循 [Conventional Commits 规范](https://www.conventionalcommits.org/zh-hans/v1.0.0/) 填写 commit 信息。可以直接调用 `cargo zrdev commit` 来代替 `git commit` 命令。注意：这**不能代替** `git add` 命令，也就是说你需要手动将修改的部分添加到暂存提交中。

使用 `cargo clean` 来删除构建的内容。

在合并你的修改分支前，请使用 [git rebase](https://git-scm.com/docs/git-rebase) 命令将修改分支的基础设为 master 的最新提交。