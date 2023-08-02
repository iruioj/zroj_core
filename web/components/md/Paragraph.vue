<script lang="ts" setup>
import Node from "./Node.vue";
import { NodeParagraph } from "composables/api";

defineProps<{
  data: NodeParagraph;
}>();
</script>

<template>
  <p class="my-2">
    <Node v-for="c in data.children" :data="c" />
  </p>
</template>
