<script setup lang="ts">
const { data: authinfo, refresh } = useAuth()
const { info } = useMsgStore()

const onLogout = async () => {
  const res = await fetch(useRuntimeConfig().public.apiBase + "/auth/logout", {
    method: 'POST',
    credentials: 'include'
  })
  console.log(res.status, await res.text())
  await refresh()
  info('已退出登录')
  navigateTo('/')
}
</script>

<template>
  <PageContainer>
    <div class="mt-8 mb-4 text-2xl text-brand font-medium">{{ authinfo?.username }}</div>
    <div class="my-2">
      <UBtn @click="onLogout">Logout</UBtn>
    </div>
  </PageContainer>
</template>