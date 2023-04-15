<!-- 考虑到服务端和客户端的时区可能不同，设置为客户端渲染组件 -->
<script setup lang="ts">
const props = defineProps<{
  time: number;
}>();
const date = computed(() => new Date(props.time));
</script>

<template>
  <span>
    <span>
      <span>{{ date.getFullYear() }}</span
      >-<span>{{ date.getMonth() }}</span
      >-<span>{{ date.getDate() }}</span>
    </span>
    <span class="pl-1">
      <span>{{ date.getHours() }}</span
      >:<span>{{ date.getMinutes() }}</span
      >:<span>{{ date.getSeconds() }}</span>
    </span>
    <sup>UTC+{{ -date.getTimezoneOffset() / 60 }}</sup>
  </span>
</template>
