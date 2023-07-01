<script setup lang="ts">
defineProps<{
  data: any;
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
    <SectionContainer title="题目信息">
      <ul class="px-2 py-1">
        <li v-if="data.meta.timeLimit != undefined" class="flex py-1">
          <div>时间限制：</div>
          <PlankTime :seconds="data.meta.timeLimit" />
        </li>
        <li v-if="data.meta.memoryLimit != undefined" class="flex py-1">
          <div>空间限制：</div>
          <div><RadixNum :num="data.meta.memoryLimit" :base="24" /> bit</div>
        </li>
        <li v-if="data.meta.io != undefined" class="flex py-1">
          <div>输入输出方式：</div>
          <div v-if="data.meta.io === 'standard'">标准输入输出</div>
        </li>
      </ul>
    </SectionContainer>

    <SectionContainer title="题目描述">
      <div class="px-1 text-md leading-7">
        <p class="my-4">
          给一个 Trie，求每个点到根的路径表示的字符串的最长 Border。给一个
          Trie，求每个点到根的路径表示的字符串的最长 Border。给一个
          Trie，求每个点到根的路径表示的字符串的最长 Border。给一个
          Trie，求每个点到根的路径表示的字符串的最长 Border。给一个
          Trie，求每个点到根的路径表示的字符串的最长 Border。
        </p>
        <p class="my-4">
          给一个 Trie，求每个点到根的路径表示的字符串的最长 Border。给一个
          Trie，求每个点到根的路径表示的字符串的最长 Border。给一个
          Trie，求每个点到根的路径表示的字符串的最长 Border。给一个
          Trie，求每个点到根的路径表示的字符串的最长 Border。给一个
          Trie，求每个点到根的路径表示的字符串的最长 Border。
        </p>
        <p class="my-4">
          给一个 Trie，求每个点到根的路径表示的字符串的最长 Border。给一个
          Trie，求每个点到根的路径表示的字符串的最长 Border。给一个
          Trie，求每个点到根的路径表示的字符串的最长 Border。给一个
          Trie，求每个点到根的路径表示的字符串的最长 Border。给一个
          Trie，求每个点到根的路径表示的字符串的最长 Border。
        </p>
        <p class="my-4">N 和字符集大小均不超过 1,000,000。</p>
      </div>
    </SectionContainer>

    <SectionContainer title="输入格式">
      <div class="px-1">
        <p class="my-4">第一行一个整数 N 表示结点个数。</p>

        <p class="my-4">
          接下来 N-1 行每行三个整数表示一条树边和这条边上的字符。
        </p>
      </div>
    </SectionContainer>

    <SectionContainer title="输出格式">
      <div class="px-1">
        <p class="my-4">一行 N 个整数表示答案。</p>
      </div>
    </SectionContainer>

    <SectionContainer title="样例">
      <div
        v-for="(sample, id) in samples"
        :key="id"
        class="px-1 my-2 mb-4 grid sm:grid-cols-2"
      >
        <div
          class="group p-2 hover:bg-black/[0.14] transition-colors cursor-pointer"
          @click="copyText(sample.input)"
        >
          <div
            class="text-brand-secondary mb-2 border-b border-dashed flex justify-between"
          >
            <div class="text-lg">样例读入 #{{ id + 1 }}</div>
            <div class="text-slate-500 invisible group-hover:visible">
              点击复制
            </div>
          </div>
          <pre class="">{{ sample.input }}</pre>
        </div>
        <div
          class="group p-2 hover:bg-black/[0.14] transition-colors cursor-pointer"
          @click="copyText(sample.output)"
        >
          <div
            class="text-brand-secondary mb-2 border-b border-dashed flex justify-between"
          >
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
