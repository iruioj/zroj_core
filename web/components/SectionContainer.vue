<!-- SectionContainer 用于显示一个段落，类似没有边框的 card -->
<script lang="ts" setup>
defineProps<{ title: string }>();
</script>

<template>
  <div class="">
    <div
      class="hover:before:text-brand before:text-brand/0 before:content-['¶'] before:absolute before:transition-colors relative before:left-[-16px] px-1 py-2 border-b-2 dark:border-brand border-brand border-dashed text-brand font-bold text-lg"
    >
      {{ title }}
    </div>
    <div>
      <slot></slot>
    </div>
  </div>
</template>
