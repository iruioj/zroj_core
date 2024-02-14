import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// console.log("NODE_ENV=", process.env.NODE_ENV);

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  devtools: { enabled: true, },
  modules: ["@nuxt/ui", "nuxt-icons", "@pinia/nuxt"],

  css: ['~/assets/main.css'],

  runtimeConfig: {
    // The private keys which are only available server-side
    apiSecret: "123",
    // Keys within public are also exposed client-side
    public: {
      // in the same origin, thus omit host.
      apiBase: "/api",
      // apiBase: "http://api.zroj.tst",
    },
  },

  devServer: {
    port: 3456,
    host: "127.0.0.1",
  },

  postcss: {
    plugins: {
      tailwindcss: {},
      autoprefixer: {},
    },
  },

  vite: {
    plugins: [wasm(), topLevelAwait()],
    server: {
      strictPort: true,
      hmr: {
        port: 3456,
        protocol: 'ws',
        host: 'localhost',
      },
    },
  },
  hooks: {
    'vite:extendConfig'(viteInlineConfig, env) {
      viteInlineConfig.server = {
        ...viteInlineConfig.server,
        hmr: {
          protocol: 'ws',
          host: 'localhost',
        },
      }
    },
  },


  typescript: {
    tsConfig: {
      compilerOptions: {
        moduleResolution: "bundler",
      },
    },

  },
});