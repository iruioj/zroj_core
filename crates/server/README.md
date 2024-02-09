# zroj_core/server

This crate builds the backend and judging server for ZROJ. In the future, it is
able to configurate the server to make it a pure judging server or a pure
resource backend.

This crate is divided roughly into three modules (see documentation of each module below).

## Development

To start the development server:

```bash
cargo run --bin test_all -- nocapture
```

To generate backend REST api types for frontend:

```bash
cargo run --bin gen_docs -- nocapture > web/composables/api.ts
```