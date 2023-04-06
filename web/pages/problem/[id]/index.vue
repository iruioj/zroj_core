<script setup lang="ts">
defineProps<{
  data: any;
}>();
</script>
<template>
  <div>
    <SectionContainer title="题目信息">
      <ul class="px-2 py-1">
        <li v-if="data.meta.timeLimit != undefined" class="flex py-1">
          <div class="font-bold">时间限制：</div>
          <PlankTime :seconds="data.meta.timeLimit" />
        </li>
        <li v-if="data.meta.memoryLimit != undefined" class="flex py-1">
          <div class="font-bold">空间限制：</div>
          <div>{{ data.meta.memoryLimit }} bit</div>
        </li>
        <li v-if="data.meta.io != undefined" class="flex py-1">
          <div class="font-bold">输入输出方式：</div>
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
  </div>
</template>
