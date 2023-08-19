import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// console.log("NODE_ENV=", process.env.NODE_ENV);

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: ["@nuxtjs/tailwindcss", "nuxt-icons", "@pinia/nuxt"],

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

  devServer: {},

  vite: {
    plugins: [wasm(), topLevelAwait()],
  },

  devtools: {
    enabled: true,
  },
});
