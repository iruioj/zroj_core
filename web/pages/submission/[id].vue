<script setup lang="ts">
const route = useRoute();
const { data, refresh } = await useAPI().submission.detail.get.use({
  sid: parseInt(route.params.id as string),
});
const onRejudge = async () => {
  const dat = data.value;
  if (dat) {
    const payload = new FormData();
    payload.append("sid", dat.info.meta.id.toString());
    await useAPI().problem.submit.post.fetch(payload)
    console.log('rejudge posted!')
  }
}
const onRefresh = async () => {
  await refresh()
}
</script>

<template>
  <PageContainer>
    <div>
      <table v-if="data" class="border-collapse w-full my-2 text-sm sm:text-md border border-table">
        <thead>
          <tr class="text-brand">
            <th class="border py-1 w-20">ID</th>
            <th class="border py-1 text-left px-1">Verdict</th>
            <th class="border py-1 px-1">Author</th>
            <th class="border py-1 px-1">Lang</th>
            <th class="border py-1">Time</th>
            <th class="border py-1">Memory</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td class="border py-1 w-20 text-center">#1</td>
            <td class="border py-1 text-left px-1">Wrong Answer</td>
            <td class="border py-1 px-1 text-center">
              {{ data.info.meta.username }}
            </td>
            <td class="border py-1 px-1 text-center">
              {{ data.info.meta.lang }}
            </td>
            <td class="border py-1 text-center">
              <span v-if="data.info.meta.time">{{ data.info.meta.time }}ms</span>
            </td>
            <td class="border py-1 text-center">
              <span v-if="data.info.meta.memory">{{ (data.info.meta.memory / 1e6).toFixed(3) }}MB</span>
            </td>
          </tr>
        </tbody>
      </table>
      <SectionContainer title="详细信息">
        <div class="mt-2">
          <template v-if="data?.info.report?.pre">
            <ReportTestset title="Pretests" :data="data.info.report.pre" />
          </template>
          <template v-if="data?.info.report?.data">
            <ReportTestset title="Tests" :data="data.info.report.data" />
          </template>
          <template v-if="data?.info.report?.extra">
            <ReportTestset title="Extra Tests" :data="data.info.report.extra" />
          </template>
        </div>
      </SectionContainer>
    </div>

    <SectionContainer v-if="data" title="源代码">
      <div v-for="(val, key) in data.info.raw" :key="key">
        <div class="py-2">
          <span class="font-bold">{{ key }}</span> ({{ val.file_type }})
        </div>
        <CodeBlock :raw="val.source" lang="cpp" />
      </div>
    </SectionContainer>

    <UButtonGroup>
    <UButton @click="onRejudge">Rejudge</UButton>
    <UButton @click="onRefresh">Refresh</UButton>
    </UButtonGroup>
  </PageContainer>
</template>
