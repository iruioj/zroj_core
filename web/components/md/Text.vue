<script lang="ts" setup>
import { NodeText } from "composables/api";

const props = defineProps<{
  data: NodeText;
}>();
</script>

<template>
  {{ data.value }}
</template>
