<script setup lang="ts">
import { ProblemStatementGetReturn } from 'composables/api';

defineProps<{
  data: ProblemStatementGetReturn | null;
}>();

const { error, info } = useMsgStore();
const copyText = (s: string) => {
  try {
    navigator.clipboard.writeText(s);
    info("复制成功");
  } catch (e: any) {
    error("Copy Failed!");
  }
};

const samples = [
  {
    input: `275.6 11.9 27.4 2.8 2
102.0 2.9
220.0 2.2`,
    output: `26.95`,
  },
  {
    input: `5
at
touch
cheat
choose
tact
a`,
    output: `23`,
  },
  {
    input: `5 3
1 3
1 3
1 4`,
    output: `4 3 2 1 5`,
  },
];
</script>
<template>
  <div>
    <SectionContainer title="题目信息" v-if="data">
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

    <SectionContainer title="样例">
      <div v-for="(sample, id) in samples" :key="id" class="px-1 my-2 mb-4 grid sm:grid-cols-2">
        <div class="group p-2 hover:bg-black/[0.14] transition-colors cursor-pointer" @click="copyText(sample.input)">
          <div class="text-brand-secondary mb-2 border-b border-dashed flex justify-between">
            <div class="text-lg">样例读入 #{{ id + 1 }}</div>
            <div class="text-slate-500 invisible group-hover:visible">
              点击复制
            </div>
          </div>
          <pre class="">{{ sample.input }}</pre>
        </div>
        <div class="group p-2 hover:bg-black/[0.14] transition-colors cursor-pointer" @click="copyText(sample.output)">
          <div class="text-brand-secondary mb-2 border-b border-dashed flex justify-between">
            <div class="text-lg">样例输出 #{{ id + 1 }}</div>
            <div class="text-slate-500 invisible group-hover:visible">
              点击复制
            </div>
          </div>
          <pre class="">{{ sample.output }}</pre>
        </div>
      </div>
    </SectionContainer>
  </div>
</template>
