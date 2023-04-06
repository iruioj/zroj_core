<script setup lang="ts">
const source = ref("");
</script>
<template>
  <textarea
    v-model="source"
    class="border border-slate-400 w-full font-mono p-2 h-128 outline-brand rounded"
    placeholder="填写源代码"
  ></textarea>
</template>
