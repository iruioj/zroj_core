<script lang="ts" setup>
const onSubmit = (payload: string) => console.log(payload)
const value = ref(`#include<iostream>

int main() {
  int a, b;
  std::cin >> a >> b;
  std::cout << a + b << std::endl;
  return 0;
}
`)
const lang = ref('plain')

const langs = [
  {
    title: "C++14",
    value: "gnu-cpp14",
    editorlang: 'cpp',
  },
  {
    title: "C++14 (O2)",
    value: "gnu-cpp14-o2",
    editorlang: 'cpp',
  },
  {
    title: "Python",
    value: "python3",
    editorlang: 'python',
  },
];
const onChangeLang = (item: Pick<typeof langs[0], 'title' | 'value'>) => {
  console.log(item)
  lang.value = langs.find(o => o.value == item.value)!.editorlang
}

</script>

<template>
  <PageContainer>
    <div class="mt-8 mb-4 text-2xl text-brand font-medium">自定义测试</div>
    <div class="flex my-2">
      <InputSelect :items="langs" placeholder="选择语言" class="w-32" @change="onChangeLang" />
      <UBtn class="mx-2" @click="onSubmit">提交</UBtn>
    </div>
    <div class="sm:hidden">请使用电脑/调大窗口尺寸以使用自定义测试</div>
    <MonacoEditor class="invisible sm:visible h-128 border border-slate-400 w-full" v-model="value" :lang="lang" />
  </PageContainer>
</template>
