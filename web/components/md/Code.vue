<script lang="ts" setup>
import CodeBlock from "../CodeBlock.vue";
import { NodeCode } from "composables/api";

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
