<script lang="ts" setup>
import "katex/dist/katex.min.css";
import katex from "katex";

import { InlineMath } from "../../composables/markdown_types";

defineProps<{
  data: InlineMath;
}>();
</script>

<template>
  <span v-html="katex.renderToString(data.value)"></span>
</template>
