// https://nuxt.com/docs/api/configuration/nuxt-config
console.log("NODE_ENV=", process.env.NODE_ENV);
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
});
