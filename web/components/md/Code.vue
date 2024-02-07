<script lang="ts" setup>
import type { NodeCode } from "@/composables/api";
import CodeBlock from "../CodeBlock.vue";

defineProps<{
  data: NodeCode;
  copyable?: boolean;
}>();
</script>

<template>
  <CodeBlock
    :raw="data.value"
    :lang="data.lang || ''"
    :meta="data.meta || ''"
    :copyable="copyable"
  />
</template>
