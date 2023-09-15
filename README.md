# ZROJ Core

本项目包含了 ZROJ 的所有核心库。

This project is currently under active development.

Diesel:

```bash
diesel migration run/redo
diesel print-schema > server/src/data/mysql/schema.rs
```

## Testing

```bash
# clear database
diesel migration redo -n 5
# generate api types
cargo run --bin gen_docs -- nocapture > web/composables/api.ts
# start test server
cargo run --bin test_all -- nocapture
```

## Formatting and Linting

```bash
cargo clippy --fix --allow-dirty --all-features -- --allow "clippy::type_complexity"
cargo fmt
```