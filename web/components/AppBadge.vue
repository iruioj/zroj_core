<script setup lang="ts">
import type { Component } from 'vue'

defineProps<{
  icon: Component
}>()
</script>

<template>
  <div class="w-full flex">
    <div class="px-2 py-0.5 border border-r-0 border-brand rounded-l-lg">
      <component :is="icon" class="w-5 h-6" />
    </div>
    <div
      class="grow px-2 py-0.5 border border-brand rounded-r-lg text-brand"
    >
      <slot></slot>
    </div>
  </div>
</template>
