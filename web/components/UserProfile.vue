<script setup lang="ts">
const props = defineProps<{
  username: string;
}>();

const { data: profile } = await useAPI().user.get.use({ username: props.username });
</script>

<template>
  <div>
    <img v-if="profile" :src="'/api/user/gravatar?email=' + profile.email" />
    <pre>{{ profile }}</pre>
  </div>
</template>
