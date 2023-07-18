# ZROJ web

https://dev.to/tao/adding-eslint-and-prettier-to-nuxt-3-2023-5bg

开发环境：

```
pnpm: 8.6
node: 18.16
wasm-pack 0.12.1
```

# 「更新」本地开发

首先安装依赖：

```bash
pnpm i
```

然后使用 wasm-pack 在项目根目录构建 passwd 包：

```bash
wasm-pack build --features wasm
```

如果卡住，可以进行终端代理。

启动开发服务器：

```bash
pnpm dev
```

# 在本地进行 HTTPS 开发

首先安装 mkcert

```bash
mkcert -install # 安装本地 CA (require sudo)
mkcert localhost # 创建证书
```

然后就可以前后端分别配置

# 密码哈希

为了前后端统一，采用 rust 库，前端 wasm 调用，开发时需要在 passwd 模块目录下执行

```bash
wasm-pack build --features wasm
```

然后在 web 目录下执行

```bash
yarn
```

更新依赖。

# 后端 API 类型标注

在项目根目录下执行

```bash
cargo run --bin gen_docs -- nocapture > web/composables/api.ts
```
