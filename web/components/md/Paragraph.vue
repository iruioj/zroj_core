<script lang="ts" setup>
import { NodeParagraph } from "composables/api";
import Node from "./Node.vue";

defineProps<{
  data: NodeParagraph;
}>();
</script>

<template>
  <p class="my-2">
    <Node v-for="c in data.children" :data="c" />
  </p>
</template>
