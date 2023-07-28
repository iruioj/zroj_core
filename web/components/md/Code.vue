<script lang="ts" setup>
import { Code } from "../../composables/markdown_types";
import CodeBlock from "../CodeBlock.vue";

defineProps<{
  data: Code;
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
