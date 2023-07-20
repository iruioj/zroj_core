<script setup lang="ts">
import hljs from "highlight.js/lib/core";
import cpp from "highlight.js/lib/languages/cpp";
import plain from "highlight.js/lib/languages/plaintext";
import "highlight.js/styles/github.css";

hljs.registerLanguage("cpp", cpp);
hljs.registerLanguage("", plain);
hljs.registerLanguage("plain", plain);

const props = defineProps<{
  raw: string;
  lang: string;
}>();

const rendered = hljs.highlight(props.raw, { language: props.lang });
</script>

<template>
  <pre
    class="w-full text-sm p-2 mb-4 rounded border border-table bg-black/[0.04] overflow-x-auto"
    v-html="rendered.value"
  ></pre>
</template>
