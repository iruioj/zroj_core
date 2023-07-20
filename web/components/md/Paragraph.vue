<script lang="ts" setup>
import { Paragraph } from '../../composables/markdown_types';
import Node from './Node.vue';

defineProps<{
    data: Paragraph
}>();

</script>

<template>
    <p class="my-2">
        <Node v-for="c in data.children" :data="c" />
    </p>
</template>