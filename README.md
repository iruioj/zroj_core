# ZROJ Core

本项目包含了 ZROJ 的所有核心库。

This project is currently under active development.

recommand VSCode extension: `mtxr.sqltools`

Diesel:

```bash
diesel migration run/redo
diesel print-schema > server/src/data/mysql/schema.rs
```

## Testing

```bash
# clear database
diesel migration redo -n 5
```

## Formatting and Linting

```bash
cargo clippy --fix --allow-dirty --all-features -- --allow "clippy::type_complexity"
cargo fmt
```

## Add Hooks before commit

editing `.git/hooks/pre-commit`:

```sh
#!/bin/zsh

cargo clippy --fix --allow-dirty --all-features -- --allow "clippy::type_complexity"
cargo fmt
```

## Document Generation

```sh
cargo doc --no-deps # generate classic rust docs
```