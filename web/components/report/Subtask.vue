<script setup lang="ts">
defineProps<{
  id: string;
  title: string;
  data: SubtaskReport;
  expand?: boolean;
}>();

const cur = useSubmExpandID();
</script>

<template>
  <div class="border border-theme rounded">
    <ReportMeta
      :id="id"
      :title="title"
      :meta="{
        ...data.meta,
        score_rate: data.meta.score_rate * data.total_score,
      }"
    />
    <TransitionCollapse>
      <div v-if="expand || cur.startsWith(id)" class="border-t border-theme">
        <template v-for="(task, index) in data.tasks">
          <div
            v-if="!task"
            class="border border-theme rounded mx-2 mb-2 first:mt-2"
          >
            Skipped
          </div>
          <ReportTask
            v-else
            :id="`${id}.t-${index}`"
            :title="`Task #${index + 1}`"
            :data="task"
            class="mx-2 mb-2 first:mt-2"
          />
        </template>
      </div>
    </TransitionCollapse>
  </div>
</template>
