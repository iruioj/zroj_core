<script setup lang="ts">
const config = useRuntimeConfig()
useHead({
  title: config.debug == 'production' ? 'Zhengrui Online Judge' : 'ZROJ (dev)'
})
</script>
<template>
  <NuxtLayout>
    default layout
  </NuxtLayout>
</template>
