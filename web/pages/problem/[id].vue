<!-- 题目描述页面 -->
<script setup lang="ts">
const id = computed(() => parseInt(useRoute().params.id as string));
const { data } = await useAPI().problem.statement.get.use({ id: id.value });

watch(data, (val) => {
  if (val) {
    useHead({
      title: val.title,
    });
  }
});
</script>

<template>
  <PageContainer>
    <div v-if="data" class="mt-8 mb-4 flex">
      <div class="grow text-2xl text-brand">
        #{{ id }} {{ data.title }}
      </div>
      <RouterTabsBar
        class="print:hidden"
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
          {
            title: '管理',
            key: 'problem-id-manage',
            link: '/problem/' + $route.params.id + '/manage',
          },
        ]"
      />
    </div>

    <NuxtPage :data="data" :pid="id" />
  </PageContainer>
</template>
