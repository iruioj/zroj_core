<script setup lang="ts">
const { data: authinfo, refresh } = await useAuth();
const { info, error } = useMsgStore();

const onLogout = async () => {
  const res = await fetch(useRuntimeConfig().public.apiBase + "/auth/logout", {
    method: "POST",
    credentials: "include",
  });
  if (res.ok) {
    await refresh();
    info("已退出登录");
    navigateTo("/");
    return;
  }
  error(await res.text());
};

</script>

<template>
  <PageContainer>
    <div class="mt-8 mb-4 text-2xl text-brand font-medium">
      {{ authinfo?.username }}
    </div>
    <UserProfile v-if="authinfo" :username="authinfo.username" />
    <div class="my-2">
      <UBtn @click="onLogout">Logout</UBtn>
      <NuxtLink to="/user/me/edit"><UBtn class="mx-1">Edit</UBtn></NuxtLink>
    </div>
  </PageContainer>
</template>
