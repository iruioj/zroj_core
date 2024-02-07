<script lang="ts" setup>
import type { NodeRoot } from "@/composables/api";
import Node from "./Node.vue";

defineProps<{
  data: NodeRoot;
}>();
</script>

<template>
  <Node v-for="c in data.children" :data="c" />
</template>
