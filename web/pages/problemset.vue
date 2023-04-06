<script lang="ts" setup>
const data = {
  page: {
    cur: 2,
    totalPage: 26,
    isFirst: false,
    isLast: false,
  },
  problems: [
    {
      id: 1,
      title: "A + B Problem",
      link: "/problem/1",
      totalLikes: 407, // likes - dislikes
      like: true,
      dislike: false,
      accepted: true, // false or undefined
    },
    {
      id: 2,
      title: "Quine",
      link: "/problem/2",
      totalLikes: 167, // likes - dislikes
      like: false,
      dislike: false,
      accepted: false,
    },
    {
      id: 3,
      title: "【17 普及 1】阿卡的萝卜",
      link: "/problem/3",
      totalLikes: 15, // likes - dislikes
      like: false,
      dislike: false,
      accepted: undefined,
    },
  ],
};
</script>
<template>
  <PageContainer>
    <SectionContainer title="题目集">
      <div class="p-2">
        <table class="w-full">
          <thead>
            <TableHeaderRow>
              <th class="pb-2">ID</th>
              <th class="pb-2 text-left">题目</th>
              <th class="pb-2">评价</th>
            </TableHeaderRow>
          </thead>
          <tbody>
            <TableRow v-for="p in data.problems" :key="p.id">
              <td class="text-center py-2">{{ p.id }}</td>
              <td class="py-2">
                <TextLink :to="p.link">{{ p.title }}</TextLink>
              </td>
              <td class="py-2 text-left w-32">
                <button>好评</button>
                <button class="mx-1">差评</button>
                {{ p.totalLikes }}
              </td>
            </TableRow>
          </tbody>
        </table>
        <div class="flex justify-center">
          <div v-if="!data.page.isFirst"><button>上一页</button></div>
          <div class="mx-4">
            第 {{ data.page.cur }} 页 / 共 {{ data.page.totalPage }} 页
          </div>
          <div v-if="!data.page.isLast"><button>下一页</button></div>
        </div>
      </div>
    </SectionContainer>
  </PageContainer>
</template>
