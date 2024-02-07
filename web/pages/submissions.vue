<script setup lang="ts">
import type { SubmMetasQuery, FileType } from "@/composables/api";

const route = useRoute();
// computed from url query, thus not reactive
const query = computed<SubmMetasQuery>(() => ({
  max_count: 15,
  offset: parseInt(route.query.offset as string) || 0,

  pid: parseInt(route.query.pid as string) || undefined,
  uid: parseInt(route.query.uid as string) || undefined,
  lang: (route.query.lang as FileType) || undefined,
}));

const { data, fetching } = await useAPI().submission.metas.get.use(query);
</script>

<template>
  <PageContainer>
    <pre>{{ data }}</pre>
  </PageContainer>
</template>
