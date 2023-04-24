<script setup lang="ts">
const username = useUsername()

const onLogout = () => {
  throw new Error('not implemented')
  // useCookie("username").value = ''
  // username.value = ''
}
</script>

<template>
  <PageContainer>
    <div class="mt-8 mb-4 text-2xl text-brand font-medium">{{ username }}</div>
    <div class="my-2">
      <UBtn @click="onLogout">Logout</UBtn>
    </div>
  </PageContainer>
</template>