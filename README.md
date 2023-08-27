# ZROJ Core

本项目包含了 ZROJ 的所有核心库。

This project is currently under active development.

## VSCode Config

```json
{
  "editor.tabSize": 4,
  "rust-analyzer.check.features": [
    "server/mysql"
  ],
  "rust-analyzer.cargo.cfgs": {
    "feature": "mysql"
  },
  "rust-analyzer.cargo.features": [
    "server/mysql"
  ]
}
```