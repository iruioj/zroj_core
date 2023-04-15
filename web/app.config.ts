export default defineAppConfig({
  title:
    process.env.NODE_ENV === "production"
      ? "Zhengrui Online Judge"
      : "ZROJ (dev)",
});
