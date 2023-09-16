<script lang="ts" setup>
import type { TaskReport } from "~/composables/api";

const { error } = useMsgStore();

const value = ref(`#include<iostream>

int main() {
  int a, b;
  std::cin >> a >> b;
  std::cout << a + b << std::endl;
  return 0;
}
`);
const inp = ref("1 2");
const lang = ref<(typeof langs)[0] | null>(null);

const langs = [
  {
    title: "C++14 (O2)",
    value: "gnu_cpp14_o2",
    editorlang: "cpp",
  },
  {
    title: "Python",
    value: "python3",
    editorlang: "python",
  },
];

const isJudging = useState("oneoff_is_judging", () => false);
const judgeResult = useState<TaskReport | null>("oneoff_report", () => null);

const onSubmit = async () => {
  const data = new FormData();
  const srcFile = new File([value.value], `main.${lang.value!.value}.cpp`);
  const inpFile = new File([inp.value], `input.txt`);

  data.append("source", srcFile);
  data.append("input", inpFile);

  try {
    await useAPI().custom_test.post.fetch(data);
    isJudging.value = true;
    const queryResult = async (): Promise<TaskReport> => {
      const data = await useAPI().custom_test.get.fetch();
      if (!data.result) {
        return new Promise((resolve, reject) => {
          setTimeout(() => {
            // console.debug("try again");
            queryResult().then(resolve).catch(reject);
          }, 1000);
        });
      } else {
        return data.result;
      }
    };

    const res = await queryResult();
    judgeResult.value = res;
  } catch (e) {
    error((e as any).data); // actually as FetchError
  }
  isJudging.value = false;
};
</script>

<template>
  <PageContainer>
    <div class="mt-8 mb-4 text-2xl text-brand font-medium">自定义测试</div>
    <div class="flex my-2">
      <InputSelect
        v-model="lang"
        :items="langs"
        placeholder="选择语言"
        class="w-32"
      />
      <UBtn class="mx-2" @click="onSubmit">提交</UBtn>
    </div>
    <textarea
      ref="sourceRef"
      v-model="value"
      class="bg-back border border-slate-400 w-full overflow-y-auto font-mono p-2 h-96 outline-brand rounded"
    ></textarea>
    <div class="my-1 text-secondary">标准读入</div>
    <textarea
      ref="sourceRef"
      v-model="inp"
      class="bg-back border border-slate-400 w-full overflow-y-auto font-mono p-2 h-32 outline-brand rounded"
    ></textarea>
    <div v-if="isJudging">评测中...</div>
    <TaskReportView v-else-if="judgeResult" :data="judgeResult" />
  </PageContainer>
</template>
