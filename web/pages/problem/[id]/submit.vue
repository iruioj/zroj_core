<script setup lang="ts">
defineProps<{
  data: any;
}>();
const r = useRoute();

if (r.name === "problem-id-submit") {
  // 默认跳转到提交源代码，后面可以修改为记忆化
  navigateTo({
    path: "/problem/" + r.params.id + "/submit/source",
  });
}
</script>

<template>
  <div>
    <div class="flex mt-2 mb-4">
      <div>选择语言</div>
      <button class="px-2">提交</button>
      <div class="grow"></div>
      <RouterTabsBar
        :items="[
          {
            title: '提交源代码',
            key: 'problem-id-submit-source',
            link: '/problem/' + $route.params.id + '/submit/source',
          },
          {
            title: '提交文件',
            key: 'problem-id-submit-file',
            link: '/problem/' + $route.params.id + '/submit/file',
          },
        ]"
      />
    </div>

    <NuxtPage />
  </div>
</template>
