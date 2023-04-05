<!-- 题目描述页面 -->
<script setup lang="ts"></script>

<template>
  <div>
    <div class="w-[700px] m-auto">
      <div class="mt-8 mb-4 text-4xl text-brand font-medium">
        #{{ $route.params.id }} 树上 KMP
      </div>

      <AppButtonGroup class="my-4">
        <AppButton>提交</AppButton>
        <AppButton>统计</AppButton>
        <AppButton>讨论</AppButton>
        <AppButton>题解</AppButton>
      </AppButtonGroup>

      <div class="flex">
        <div class="mr-1 oj-tag">1.854x10<sup>43</sup> Planck Time</div>
        <div class="mr-1 oj-tag">2147483648 bit</div>
        <div class="mr-1 oj-tag">Standard IO</div>
      </div>

      <div class="my-4 text-2xl font-medium">题目描述</div>
      <div class="text-md leading-7">
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

      <!-- <CodeEditor v-model="content" class="rounded my-4 text-base" /> -->
    </div>
  </div>
</template>
<style>
.oj-tag {
  @apply px-2 rounded bg-brand-light text-brand;
}
</style>
