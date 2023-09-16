<script lang="ts" setup>
import { NodeTwoColumns } from "composables/api";
import Node from "./Node.vue";
import Code from "./Code.vue";

defineProps<{
  data: NodeTwoColumns;
}>();
</script>

<template>
  <div class="flex">
    <div class="mr-1 w-full">
      <Code v-if="data.left.type == 'code'" :data="data.left" copyable />
      <Node v-else :data="data.left" />
    </div>
    <div class="ml-1 w-full">
      <Code v-if="data.right.type == 'code'" :data="data.right" copyable />
      <Node v-else :data="data.right" />
    </div>
  </div>
</template>
