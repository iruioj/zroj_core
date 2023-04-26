# ZROJ web

https://dev.to/tao/adding-eslint-and-prettier-to-nuxt-3-2023-5bg

开发环境：

```
yarn: 3.4.1
node: 18.16.0
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
