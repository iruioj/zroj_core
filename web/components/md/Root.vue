<script lang="ts" setup>
import Node from "./Node.vue";
import type { NodeRoot } from "@/composables/api";

defineProps<{
  data: NodeRoot;
}>();
</script>

<template>
  <Node v-for="c in data.children" :data="c" />
</template>
