<!-- 题目描述页面 -->
<script setup lang="ts">
const data = {
  id: 1,
  title: "树上 KMP",
  meta: {
    timeLimit: 1,
    memoryLimit: 2147483648,
    io: "standard",
  },
};
</script>

<template>
  <PageContainer>
    <div class="mt-8 mb-4 flex">
      <div class="grow text-2xl text-brand">
        #{{ $route.params.id }} {{ data.title }}
      </div>
      <RouterTabsBar
        :items="[
          {
            title: '题面',
            key: 'problem-id',
            link: '/problem/' + $route.params.id,
          },
          {
            title: '提交',
            key: 'problem-id-submit',
            link: '/problem/' + $route.params.id + '/submit',
          },
          {
            title: '统计',
            key: 'problem-id-statics',
            link: '/problem/' + $route.params.id + '/statics',
          },
        ]"
      />
    </div>

    <NuxtPage :data="data" />
  </PageContainer>
</template>
