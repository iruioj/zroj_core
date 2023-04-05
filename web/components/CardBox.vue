<!-- 卡片布局 -->
<script lang="ts" setup>
</script>

<template>
  <div class="border border-black/20 rounded">
    <div class="p-2 border-b border-black/20">
      <slot name="header"></slot>
    </div>
    <div class="p-2">
      <slot name="body"></slot>
    </div>
  </div>
</template>