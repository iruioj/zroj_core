<script lang="ts" setup>
import type { NodeInlineCode } from "@/composables/api";

defineProps<{
  data: NodeInlineCode;
}>();
</script>

<template>
  <code>{{ data.value }}</code>
</template>
