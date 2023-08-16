<script lang="ts" setup>
const route = useRoute();
const router = useRouter();
const query = computed(() => ({
  max_count: 15,
  offset: parseInt(route.query.offset as string) || 0,
  pattern: (route.query.pattern as string) || "",
}));
const pattern = ref((route.query.pattern as string) || "");

const { data, fetching } = await useAPI().problem.metas.get.use(query);

const toPrevPage = () => {
  let offset = query.value.offset - query.value.max_count;
  if (offset < 0) offset = 0;
  router.push(`?offset=${offset}&pattern=${query.value.pattern}`);
};
const toNextPage = () => {
  const offset = query.value.offset + query.value.max_count;
  router.push(`?offset=${offset}&pattern=${query.value.pattern}`);
};
const onSearch = () => {
  if (pattern.value !== query.value.pattern) {
    router.push(`?offset=0&pattern=${pattern.value}`);
  }
};
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
        <div class="flex justify-center">
          <div class="mx-1">
            <UBtn :disable="query.offset <= 0" @click="toPrevPage">上一页</UBtn>
          </div>
          <div class="mx-1">
            <UBtn
              :disable="(data?.length || 0) < query.max_count"
              @click="toNextPage"
              >下一页</UBtn
            >
          </div>
          <form class="mx-1" @submit.prevent="onSearch">
            <InputText v-model="pattern" />
          </form>
          <div class="mx-1">
            <UBtn @click="onSearch">搜索</UBtn>
          </div>
        </div>
      </div>
    </SectionContainer>
  </PageContainer>
</template>
