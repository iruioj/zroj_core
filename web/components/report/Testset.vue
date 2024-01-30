<script setup lang="ts">
defineProps<{
  title: string;
  data: JudgeReport;
}>();
</script>

<template>
  <div class="border border-theme rounded">
    <ReportMeta :title="title" :meta="data.meta" />
    <div class="border-t border-theme p-2">
      <template v-if="data.detail.type === 'Subtask'">
        <template v-for="(subtask, index) in data.detail.tasks">
          <ReportSubtask
            v-if="subtask"
            :id="`${title}.sb-${index}`"
            :title="`Subtask #${index + 1}`"
            :data="subtask"
            class="mt-2 first:mt-0"
          />
          <div v-else class="border border-theme rounded mt-2 first:mt-0">
            skipped
          </div>
        </template>
      </template>
      <template v-else>
        <template v-for="(task, index) in data.detail.tasks">
          <ReportTask
            v-if="task"
            :id="`${title}.t-${index}`"
            :title="`Task #${index + 1}`"
            :data="task"
            class="mt-2 first:mt-0"
          />
          <div v-else class="border border-theme rounded mt-2 first:mt-0">
            skipped
          </div>
        </template>
      </template>
    </div>
  </div>
</template>
