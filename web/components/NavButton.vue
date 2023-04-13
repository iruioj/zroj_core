<script setup lang="ts">
defineProps<{
  to: string;
}>();
</script>

<template>
  <NuxtLink
    class="text-secondary hover:text-brand inline-block h-full p-2 text-sm transition duration-200 ease-in-out"
    :to="to"
  >
    <slot></slot>
  </NuxtLink>
</template>
