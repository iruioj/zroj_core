<script setup lang="ts">
import { TaskReport } from 'composables/api';

const props = defineProps<{
  data: TaskReport,
  collapse?: boolean,
}>()


const statusTitle = {
  accepted: "Accepted",
  compile_error: "Compile Error",
  custom: "Unknown Error",
  dangerous_syscall: "Dangerous System Call",
  memory_limit_exceeded: "Memory Limit Exceeded",
  output_limit_exceeded: "Output Limit Exceeded",
  partial: "Partially Accepted",
  presentation_error: "Presentation Error",
  runtime_error: "Runtime Error",
  time_limit_exceeded: "Time Limited Exceeded",
  wrong_answer: "Wrong Answer",
}
</script>

<template>
  <div class="border border-slate-400 rounded">
    <div class="flex">
      <div class="p-2">{{ statusTitle[data.meta.status.name] }}</div>
      <div class="p-2">{{ Math.round(data.meta.score * 100) }}pts</div>
      <div class="p-2">Time: {{ data.meta.time }}ms</div>
      <div class="p-2">Memory: {{ (data.meta.memory / 1e6).toFixed(3) }}MB</div>
    </div>
    <TransitionCollapse>
      <div v-if="!collapse" class="border-t border-slate-400">
        <template v-for="[name, ctnt] in data.payload" :key="name">
          <div v-if="ctnt.str.length" class="px-2 py-1">
            <div>
              <span>{{ name }}</span>
              <span class="text-secondary font-mono" v-if="ctnt.truncated"> ({{ ctnt.truncated }} characters truncated)</span>
            </div>
            <CodeBlock :raw="ctnt.str" lang="" />
          </div>
        </template>
      </div>
    </TransitionCollapse>
  </div>
</template>