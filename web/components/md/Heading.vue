<script lang="ts" setup>
import Node from "./Node.vue";
import type { NodeHeading } from "@/composables/api";

defineProps<{
  data: NodeHeading;
}>();
</script>

<template>
  <div
    v-if="data.depth == 2"
    class="hover:before:text-brand before:text-brand/0 before:content-['¶'] before:absolute before:transition-colors relative before:left-[-16px] px-1 py-2 border-b border-brand/70 border-dashed text-brand font-bold text-lg my-3"
  >
    <Node v-for="c in data.children" :data="c" />
  </div>
  <div
    v-else-if="data.depth == 3"
    class="hover:before:text-brand before:text-brand/0 before:content-['¶'] before:absolute before:transition-colors relative before:left-[-16px] px-1 py-2 text-brand font-bold text-md my-3"
  >
    <Node v-for="c in data.children" :data="c" />
  </div>
  <div
    v-else-if="data.depth == 4"
    class="hover:before:text-brand before:text-brand/0 before:content-['¶'] before:absolute before:transition-colors relative before:left-[-16px] px-1 py-2 text-brand text-md my-3"
  >
    <Node v-for="c in data.children" :data="c" />
  </div>
  <pre v-else>{{ data }}</pre>
</template>
