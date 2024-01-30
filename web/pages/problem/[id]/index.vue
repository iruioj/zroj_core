<script setup lang="ts">
import { ProblemStatementGetReturn } from "@/composables/api";

defineProps<{
  data: ProblemStatementGetReturn | null;
}>();
</script>
<template>
  <div>
    <SectionContainer v-if="data" title="题目信息">
      <ul class="px-2 py-1">
        <li class="flex py-1">
          <div>时间限制：</div>
          <PlankTime v-if="data.meta.time" :seconds="data.meta.time" />
          <div v-else>无</div>
        </li>
        <li class="flex py-1">
          <div>空间限制：</div>
          <div v-if="data.meta.memory">
            <RadixNum :num="data.meta.memory" :base="24" /> bit
          </div>
          <div v-else>无</div>
        </li>
        <li class="flex py-1">
          <div>题目类型：</div>
          <div>{{ data.meta.kind }}</div>
        </li>
      </ul>
    </SectionContainer>

    <MdNode v-if="data" :data="data.statement" />
  </div>
</template>
