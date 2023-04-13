<script setup lang="ts">
const props = defineProps<{
  num: number
  base: number
}>()

const str = computed(() => {
  let s = '', x = props.num, b = props.base
  while(x != 0) {
    let d = x % b;
    x = Math.floor(x / b);
    s = String.fromCharCode(d < 10 ? d + 48 : d + 87) + s
  }
  return s
})
</script>

<template>
  <span>({{ str }})<sub>{{ base }}</sub></span>
</template>