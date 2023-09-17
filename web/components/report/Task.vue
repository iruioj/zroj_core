<script setup lang="ts">
defineProps<{
  id: string;
  title: string;
  data: TaskReport;
  expand?: boolean;
}>();

const cur = useSubmExpandID()
</script>

<template>
  <div class="border border-theme rounded">
    <ReportMeta :id="id" :title="title" :meta="data.meta" />
    <TransitionCollapse>
      <div v-if="expand || cur === id" class="border-t border-theme">
        <template v-for="[name, ctnt] in data.payload" :key="name">
          <div v-if="ctnt.str.length" class="px-2 py-1">
            <div>
              <span>{{ name }}</span>
              <span v-if="ctnt.truncated" class="text-secondary font-mono">
                ({{ ctnt.truncated }} characters truncated)</span
              >
            </div>
            <CodeBlock :raw="ctnt.str" lang="" />
          </div>
        </template>
      </div>
    </TransitionCollapse>
  </div>
</template>
