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

const langs = [
  {
    title: "C++14",
    value: "gnu-cpp14",
  },
  {
    title: "C++14 (O2)",
    value: "gnu-cpp14-o2",
  },
  {
    title: "Python 3",
    value: "python",
  },
  {
    title: "Rust",
    value: "rust",
  },
];
</script>

<template>
  <div>
    <div class="flex mt-2 mb-4">
      <InputSelect :items="langs" placeholder="选择语言" class="w-32" />
      <UBtn class="mx-2">提交</UBtn>
      <div class="grow"></div>
      <RouterTabsBar
        :items="[
          {
            title: '代码',
            key: 'problem-id-submit-source',
            link: '/problem/' + $route.params.id + '/submit/source',
          },
          {
            title: '文件',
            key: 'problem-id-submit-file',
            link: '/problem/' + $route.params.id + '/submit/file',
          },
        ]"
      />
    </div>

    <NuxtPage />
  </div>
</template>
