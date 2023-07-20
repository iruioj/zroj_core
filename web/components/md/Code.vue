<script lang="ts" setup>
import { Code } from '../../composables/markdown_types';
import CodeBlock from '../CodeBlock.vue';

defineProps<{
  data: Code
}>();

</script>

<template>
    <CodeBlock :raw="data.value" :lang="data.lang || ''" />
</template>