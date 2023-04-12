<!-- SectionContainer 用于显示一个段落，类似没有边框的 card -->
<script lang="ts" setup>
defineProps<{ title: string }>();
</script>

<template>
  <div class="">
    <div
      class="px-1 py-2 border-b-4 border-b-brand border-double text-brand font-bold text-lg"
    >
      {{ title }}
    </div>
    <div>
      <slot></slot>
    </div>
  </div>
</template>
