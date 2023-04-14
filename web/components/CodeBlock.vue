<script setup lang="ts">
import hljs from "highlight.js/lib/core";
import cpp from "highlight.js/lib/languages/cpp";
import "highlight.js/styles/github.css";

hljs.registerLanguage("cpp", cpp);

const props = defineProps<{
  raw: string;
  lang: string;
}>();

const rendered = hljs.highlight(props.raw, { language: props.lang });
</script>

<template>
  <pre class="text-sm p-2 mb-4 rounded border border-slate-300 bg-black/[0.04] whitespace-pre-wrap" v-html="rendered.value"></pre>
</template>
