<script setup lang="ts">

const props = defineProps<{
  username: string
}>();

const { data: profile } = await useAPI("get:/user", { username: props.username })

</script>

<template>
  <div>
    <pre>{{ profile }}</pre>
  </div>
</template>