<script lang="ts" setup>
import Node from "./Node.vue";
import { NodeRoot } from "composables/api";

const props = defineProps<{
  data: NodeRoot;
}>();
</script>

<template>
  <Node v-for="c in data.children" :data="c" />
</template>
