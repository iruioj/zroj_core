<script lang="ts" setup>
// const data = {
//   page: {
//     cur: 2,
//     totalPage: 26,
//     isFirst: false,
//     isLast: false,
//   },
//   problems: genProbSet(10),
// };
const { data } = await useAPI().problem.metas.get.use({
  max_count: 20,
  min_id: 0,
  max_id: undefined,
  pattern: undefined
});

</script>
<template>
  <PageContainer>
    <SectionContainer title="题目集">
      <div class="py-2">
        <table class="w-full">
          <thead>
            <TableHeaderRow>
              <th class="pb-2">ID</th>
              <!-- <th class="w-7 pb-2"></th> -->
              <th class="pb-2 text-left">题目</th>
              <!-- <th class="pb-2">通过数</th> -->
            </TableHeaderRow>
          </thead>
          <tbody v-if="data">
            <TableRow v-for="p in data" :key="p[0]">
              <td class="text-center py-2">{{ p[0] }}</td>
              <!-- <td class="text-center py-2">
                <NuxtIcon v-if="p.accepted" class="inline-block align-middle text-brand" name="check" />
              </td> -->
              <td class="py-2">
                <TextLink :to="'/problem/' + p[0]">{{ p[1].title }}</TextLink>
              </td>
              <!-- <td class="text-center py-2">{{ p.accepts }}</td> -->
            </TableRow>
          </tbody>
        </table>
        <!-- <div class="flex justify-center">
          <div v-if="!data.page.isFirst"><button>上一页</button></div>
          <div class="mx-4">
            第 {{ data.page.cur }} 页 / 共 {{ data.page.totalPage }} 页
          </div>
          <div v-if="!data.page.isLast"><button>下一页</button></div>
        </div> -->
      </div>
    </SectionContainer>
  </PageContainer>
</template>
