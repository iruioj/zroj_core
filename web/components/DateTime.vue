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
