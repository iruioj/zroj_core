<script lang="ts" setup>
import { InlineMath } from '../../composables/markdown_types';

import 'katex/dist/katex.min.css'
import katex from 'katex'

const props = defineProps<{
  data: InlineMath
}>();

</script>

<template>
  <span v-html="katex.renderToString(data.value)"></span>
</template>