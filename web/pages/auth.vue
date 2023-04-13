<template>
  <PageContainer>
    <RouterTabsBar class="mt-8 mb-4" :items="[
      {
        title: '登录',
        key: 'auth-signin',
        link: '/auth/signin'
      },
      {
        title: '注册',
        key: 'auth-signup',
        link: '/auth/signup'
      }
    ]" />
    <div class="px-2">
      <NuxtPage />
    </div>
  </PageContainer>
</template>