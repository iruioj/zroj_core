<script setup lang="ts">
defineProps<{
  to: string;
}>();
</script>

<template>
  <NuxtLink
    :class="$route.path === to && '!text-brand after:!bg-brand'"
    class="relative inline-block text-secondary hover:text-brand inline-block h-full p-2 transition-colors
    after:bg-transparent after:absolute after:inset-x-2 after:bottom-0 after:h-[2px] hover:after:bg-brand
    after:transition-colors"
    :to="to"
  >
    <slot></slot>
  </NuxtLink>
</template>