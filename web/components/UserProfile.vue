<script setup lang="ts">

const props = defineProps<{
  username: string
}>();

const { data: profile, pending } = await useLazyAsyncData("user_profile", async () => {
  if (process.server) return;

  const res = await fetch(useRuntimeConfig().public.apiBase + "/user/" + props.username)
  // console.log(res)

  if (res.ok) {
    return await res.json();
  }
})

</script>

<template>
  <div v-if="!pending">
    <pre>{{ profile }}</pre>
  </div>
</template>