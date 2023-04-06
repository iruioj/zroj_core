<!-- 根据 router 路径改变高亮状态的标签栏 -->
<script lang="ts" setup>
const props = defineProps<{
  items: {
    title: string;
    key: string;
    link: string;
  }[];
}>();

const activeKey = computed(() => {
  const routeName = useRoute().name?.toString() || "";
  return props.items.reduce((sum, item) => {
    if (routeName.includes(item.key)) {
      // 如果是树形的 route 结构，那么越长的 key 对应越近的祖先（用于处理 tabs 中有祖先关系的情况）
      return item.key.length > sum.length ? item.key : sum;
    } else {
      return sum;
    }
  }, "");
});
</script>

<template>
  <TabsBar :items="items" :active-key="activeKey" />
</template>
