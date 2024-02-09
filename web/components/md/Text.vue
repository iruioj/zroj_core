<script lang="ts" setup>
import type { NodeText } from "@/composables/api";

defineProps<{
  data: NodeText;
}>();
</script>

<template>
  {{ data.value }}
</template>
