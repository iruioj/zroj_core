<script lang="ts" setup>
import { Root } from '../../composables/markdown_types';
import Node from './Node.vue';

const props = defineProps<{
    data: Root
}>();

</script>

<template>
    <Node v-for="c in data.children" :data="c" />
</template>