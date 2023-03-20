<script setup lang="ts">
import type { Component } from 'vue'

defineProps<{
  to: string
  icon: Component
}>()
</script>

<template>
  <NuxtLink
    class="hover:bg-brand-light h-full px-2 py-2 transition duration-200 ease-in-out"
    :to="to"
  >
    <component :is="icon" class="inline w-7 h-7 pr-1" />
    <slot></slot>
  </NuxtLink>
</template>
