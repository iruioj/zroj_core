<template>
  <div class="w-screen h-screen bg-back overflow-auto">
    <NavBar />
    <NuxtPage />
    <FootBar />
  </div>
</template>
<style>
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --theme-bg: 255 255 255;
    --theme-pri: 0 0 0;
    --theme-sec: 51 65 85;
    --theme-brand: 140 0 0;
    --theme-brand-sec: 86 0 0;
  }
  html {
    color: rgb(var(--theme-pri));
  }

  @media (prefers-color-scheme: dark) {
    :root {
      --theme-pri: 238 238 238;
      --theme-sec: 171 171 171;
      --theme-bg: 32 33 36;
      --theme-brand: 201 52 52;
      --theme-brand-sec: 202 92 92;
    }
  }
}

@layer components {
  .border-table,
  .border-table td,
  .border-table th {
    @apply border-[#e7e7e9] dark:border-[#2d2e31];

  }
  .border-theme {
    @apply border-slate-900/10 dark:border-slate-50/[0.06];
  }
  /* ... */
}
</style>
