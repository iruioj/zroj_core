<script setup lang="ts">
import type { Submission } from '../submit.vue';
const source = ref("");

const emit = defineEmits<{
  (e: 'change', payload: Submission): void
}>()

watch([source], ([source]) => {
  emit('change', { type: 'source', payload: source })
}, {
  immediate: true
})

</script>
<template>
  <textarea
    v-model="source"
    class="bg-back border border-slate-400 w-full overflow-y-auto font-mono p-2 h-128 outline-brand rounded"
    placeholder="填写源代码"
  ></textarea>
</template>
