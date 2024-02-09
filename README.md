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

First init database and generate data for testing:

```bash
cargo run --bin gen_testdata
```

Then start the dev server:

```bash
cargo run --bin test_all -- nocapture
```

CD into `crates/passwd` and run

```bash
wasm-pack build --features wasm
```

to build the `passwd` WASM package for front end.

CD into `web` and run

```bash
pnpm i
pnpm dev
```

to start the frondend dev server.

## Formatting and Linting

```bash
cargo clippy --fix --allow-dirty --all-features -- --allow "clippy::type_complexity"
cargo fmt
```

## Add Hooks before commit

This hook helps to prevent committing to the `master` branch directly.

Editing `.git/hooks/pre-commit` as:

```sh
#!/bin/zsh

branch="$(git rev-parse --abbrev-ref HEAD)"

if [ "$branch" = "master" ]; then
  echo "You can't commit directly to master branch"
  exit 1
fi
```

and make it executable.

## Document Generation

```sh
cargo doc --no-deps # generate classic rust docs
```