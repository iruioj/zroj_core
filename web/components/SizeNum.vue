<script lang="ts" setup>
defineProps<{ bytes: number }>();
</script>
<template>
  <span v-if="bytes < 1000">{{ bytes }} bytes</span>
  <span v-else-if="bytes < 1e6">{{ bytes / 1000 }} kb</span>
  <span v-else>{{ bytes / 1e6 }} mb</span>
</template>
