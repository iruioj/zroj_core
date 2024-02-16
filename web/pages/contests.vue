<script setup lang="ts">
const route = useRoute();
const router = useRouter();
// computed from url query, thus not reactive
const query = computed<CtstMetasQuery>(() => ({
  max_count: 15,
  offset: parseInt(route.query.offset as string) || 0,
  pattern: route.query.pattern as string,
}));
const pattern = ref(query.value.pattern || "");

const { data, fetching } = await useAPI().contest.metas.get.use(query);
</script>

<template>
  <PageContainer>
    <SectionContainer title="比赛列表">
      <div class="py-2">
        <table v-if="data" class="w-full text-sm hidden sm:table">
          <thead>
            <TableHeaderRow>
              <th class="px-2 pb-2 text-left">比赛</th>
              <th class="pb-2 text-center">开始时间/时长</th>
              <th class="pb-2">报名人数</th>
            </TableHeaderRow>
          </thead>
          <tbody>
            <TableRow v-for="p in data" :key="p.id">
              <td class="p-2">
                <TextLink :to="`/contest/${p.id}`">{{ p.title }}</TextLink>
              </td>
              <td class="py-2 text-center flex flex-col whitespace-nowrap">
                <DateTime :time="p.start_time" />
                <TimeElapse :elapse="p.duration" />
              </td>
              <td class="py-2 text-center">
                {{ 114514 }}
              </td>
            </TableRow>
          </tbody>
        </table>

        <ul v-if="data" class="sm:hidden">
          <li v-for="p in data" :key="p.id" class="py-2 border-b border-theme">
            <div class="text-md pb-1">
              <TextLink :to="`/contest/${p.id}`">{{ p.title }}</TextLink>
            </div>
            <div>
              <NuxtIcon name="schedule" class="inline-block align-middle" />{{
                " "
              }}
              <DateTime :time="p.start_time" />
            </div>
            <div>
              <NuxtIcon name="timer" class="inline-block align-middle" />
              <TimeElapse :elapse="p.duration" />
            </div>
            <div>
              <NuxtIcon name="group" class="inline-block align-middle" />
              {{ 114514 }}
            </div>
          </li>
        </ul>
      </div>
    </SectionContainer>
  </PageContainer>
</template>
