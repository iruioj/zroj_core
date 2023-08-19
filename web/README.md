# ZROJ web

https://dev.to/tao/adding-eslint-and-prettier-to-nuxt-3-2023-5bg

开发环境：

```
pnpm: 8.6
node: 18.16
wasm-pack 0.12.1
```

# 本地开发

首先使用 wasm-pack 在项目根目录构建 passwd 包，如果卡住，可以进行终端代理：

```bash
wasm-pack build --features wasm
```

然后安装依赖，启动开发服务器：

```bash
cd web
pnpm i
pnpm dev
```

# 在本地进行 HTTPS 开发

首先安装 mkcert

```bash
mkcert -install # 安装本地 CA (require sudo)
mkcert localhost # 创建证书
```

然后就可以前后端分别配置

# 后端 API 类型标注

在项目根目录下执行

```bash
cargo run --bin gen_docs -- nocapture > web/composables/api.ts
```
