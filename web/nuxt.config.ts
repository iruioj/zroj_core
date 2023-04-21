import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

console.log("NODE_ENV=", process.env.NODE_ENV);

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: ["@nuxtjs/tailwindcss"],
  runtimeConfig: {
    // The private keys which are only available server-side
    apiSecret: "123",
    // Keys within public are also exposed client-side
    public: {
      apiBase: "https://localhost:8080",
    },
  },
  devServer: {
    // https: {
    //   key: '../cli/src/bin/localhost-key.pem',
    //   cert: '../cli/src/bin/localhost.pem',
    // },
  },
  vite: {
    plugins: [
      wasm(),
      topLevelAwait()
    ],
  },
});
