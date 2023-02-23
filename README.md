# ZROJ Core

本项目包含了 ZROJ 的所有核心库，**不包含任何可执行文件**。

## How to Contribute

开发环境：

- 请使用 VS Code 作为编辑器，并安装 rust-analyzer 插件。
- 请掌握 Git 的基本用法，（配合 SSH key 验证的使用）。

要新建模块，请使用 `cargo new <模块名称> --lib` 命令，并在本目录下的 Cargo.toml 的 members 中添加上对应的文件夹名字。

为了降低开发难度，请大家用**中文或者英文**为自己编写的公开的函数、结构体和 traits 提供描述和注意事项。

每个模块 dependencies 应当分开管理，也就是说在对应的模块子目录下调用 `cargo` 命令。项目根目录下的 `Cargo.toml` 应当只用于公共的依赖（这些依赖需要在商讨后再确定）。

同理，每个模块的版本号也当分开管理。