<script setup lang="ts">
const { data } = await useAPI().user.edit.get(); // useAPI("get:/user/edit")
</script>

<template>
  <PageContainer>
    <div v-if="data" class="mt-8 mb-4 text-2xl text-brand font-medium">
      {{ data.username }}
    </div>
    <UserProfileEditForm v-if="data" :initdata="data" />
  </PageContainer>
</template>
