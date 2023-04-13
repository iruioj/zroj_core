<script lang="ts" setup>
const sourceRef = ref<HTMLElement | null>(null);
const code = ref("");
const submitCode = () => {
  console.log(code.value);
};

const langs = [
  {
    title: "C++14",
    value: "gnu-cpp14",
  },
  {
    title: "C++14 (O2)",
    value: "gnu-cpp14-o2",
  },
];
// https://coolors.co/gradient-maker/c70000-ec6600
</script>

<template>
  <PageContainer>
    <div class="mt-8 mb-4 text-2xl text-brand font-medium">自定义测试</div>
    <div class="flex my-2">
      <InputSelect :items="langs" placeholder="选择语言" class="w-32" />
      <UBtn class="mx-2" @click="submitCode">提交</UBtn>
    </div>
    <textarea
      ref="sourceRef"
      v-model="code"
      class="bg-back border border-slate-400 w-full overflow-y-auto font-mono p-2 h-128 outline-brand rounded"
    ></textarea>
  </PageContainer>
</template>
