<script setup lang="ts">
defineProps<{
  to: string;
}>();
</script>
<template>
  <NuxtLink
    :to="to"
    class="text-brand-secondary hover:text-brand grow underline decoration-brand/0 hover:decoration-brand underline-offset-4 transition-colors"
  >
    <slot></slot>
  </NuxtLink>
</template>
