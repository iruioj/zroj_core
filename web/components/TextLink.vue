<script setup lang="ts">
defineProps<{
  to: string;
}>();
</script>
<template>
  <NuxtLink
    :to="to"
    class="text-brand-dark grow underline decoration-brand/0 hover:decoration-brand underline-offset-4 transition ease-in-out duration-200"
  >
    <slot></slot>
  </NuxtLink>
</template>
