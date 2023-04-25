<script lang="ts" setup>
const data = {
  page: {
    cur: 2,
    totalPage: 26,
    isFirst: false,
    isLast: false,
  },
  problems: genProbSet(10),
};
</script>
<template>
  <PageContainer>
    <SectionContainer title="题目集">
      <div class="py-2">
        <table class="w-full">
          <thead>
            <TableHeaderRow>
              <th class="text-brand pb-2">ID</th>
              <th class="text-brand pb-2 text-left">题目</th>
              <th class="text-brand pb-2">评价</th>
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
