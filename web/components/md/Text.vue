<script lang="ts" setup>
import { Text } from '../../composables/markdown_types';

const props = defineProps<{
  data: Text
}>();

</script>

<template>
  {{ data.value }}
</template>