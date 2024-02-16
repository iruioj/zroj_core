<script lang="ts" setup>
import type { NodeLink, NodeText } from "@/composables/api";

defineProps<{
  data: NodeLink;
}>();

function isPdf(data: NodeLink): boolean {
  const c = data.children.at(0);
  return c?.type == "text" && (c as NodeText).value.startsWith("pdf");
}
</script>

<template>
  <iframe
    v-if="isPdf(data)"
    :src="data.url"
    style="width: 100%; height: 600px; border: none"
  ></iframe>
  <pre v-else>{{ data }}</pre>
</template>
