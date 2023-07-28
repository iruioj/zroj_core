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
  meta?: string;
  /// default: false
  copyable?: boolean;
}>();

const rendered = hljs.highlight(props.raw, { language: props.lang });

const { error, info } = useMsgStore();
const copyText = (s: string) => {
  if (props.copyable) {
    try {
      navigator.clipboard.writeText(s);
      info("复制成功");
    } catch (e: any) {
      error("Copy Failed!");
    }
  }
};
</script>

<template>
  <div class="relative group" @click="copyText(raw)">
    <pre
      :title="meta"
      :class="[
        'w-full text-sm p-2 mb-4 rounded border border-table bg-black/[0.04] overflow-x-auto',
        (copyable && 'hover:bg-black/[0.1] transition-all cursor-pointer') ||
          '',
      ]"
      v-html="rendered.value"
    ></pre>
    <div
      v-if="copyable"
      class="absolute right-1.5 top-1.5 text-black/[0.2] hidden group-hover:block"
    >
      点击复制
    </div>
  </div>
</template>
