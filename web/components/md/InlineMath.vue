<script lang="ts" setup>
import "katex/dist/katex.min.css";
import katex from "katex";
import type { NodeInlineMath } from "@/composables/api";

defineProps<{
  data: NodeInlineMath;
}>();
</script>

<template>
  <span v-html="katex.renderToString(data.value)"></span>
</template>
