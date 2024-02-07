<script lang="ts" setup>
import Node from "./Node.vue";
import type { NodeStrong } from "@/composables/api";

defineProps<{
  data: NodeStrong;
}>();
</script>

<template>
  <strong>
    <Node v-for="c in data.children" :data="c" />
  </strong>
</template>
