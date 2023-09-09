<script lang="ts" setup>
import { ProbMetasQuery } from '~/composables/api';

const route = useRoute();
const router = useRouter();
// computed from url query, thus not reactive
const query = computed<ProbMetasQuery>(() => ({
  max_count: 15,
  offset: parseInt(route.query.offset as string) || 0,
  pattern: route.query.pattern as string,
}));
const pattern = ref(query.value.pattern || "");

const { data, fetching } = await useAPI().problem.metas.get.use(query);

const toPrevPage = () => {
  let offset = query.value.offset - query.value.max_count;
  if (offset < 0) offset = 0;
  router.push({ query: { offset, pattern: pattern.value } });
  // router.push(`?offset=${offset}&pattern=${query.value.pattern}`);
};
const toNextPage = () => {
  const offset = query.value.offset + query.value.max_count;
  router.push({ query: { offset, pattern: pattern.value } });
  // router.push(`?offset=${offset}&pattern=${query.value.pattern}`);
};
const onSearch = () => {
  if (pattern.value !== query.value.pattern) {
    // router.push(`?offset=0&pattern=${pattern.value}`);
    router.push({ query: { offset: 0, pattern: pattern.value } });
  }
};
</script>
<template>
  <PageContainer>
      <div class="md:flex justify-center">
        <div class="flex py-1">
          <div class="mx-1">
            <UBtn :disable="query.offset <= 0" @click="toPrevPage">上一页</UBtn>
          </div>
          <div class="mx-1">
            <UBtn :disable="(data?.length || 0) < query.max_count" @click="toNextPage">下一页</UBtn>
          </div>
        </div>
        <div class="flex py-1">
          <form class="mx-1" @submit.prevent="onSearch">
            <InputText v-model="pattern" />
          </form>
          <div class="mx-1">
            <UBtn @click="onSearch">搜索</UBtn>
          </div>
        </div>
      </div>
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
      </div>
  </PageContainer>
</template>
