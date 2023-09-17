<script setup lang="ts">
import { red } from 'tailwindcss/colors';

const r = useRoute();

if (r.name === "problem-id-submit") {
  // 默认跳转到提交源代码，后面可以修改为记忆化
  navigateTo("/problem/" + r.params.id + "/submit/source");
}

const langs = [
  {
    title: "C++14 (O2)",
    value: "gnu_cpp14_o2",
  },
  {
    title: "Python 3",
    value: "python",
  },
];

const lang = ref<(typeof langs)[0] | null>(null);

const submission = ref<Submission | null>(null);
const onChangeSubmission = (subm: Submission) => {
  submission.value = subm;
};

const onSubmit = async () => {
  const s = submission.value;
  if (!s) return;

  const form = new FormData();
  form.append("pid", r.params.id as string);
  // append will not override existing key-value pair
  if (s.type === "source") {
    form.append(
      "files",
      new File([s.payload], `source.${lang.value!.value}.cpp`),
    );
  } else {
    form.append(
      "files",
      new File([s.payload], `source.${lang.value!.value}.cpp`),
    );
  }

  const ret = await useAPI().problem.submit.post.fetch(form);

  navigateTo('/submission/' + ret.sid)
  // console.log(ret.sid);
};
</script>

<script lang="ts">
export type Submission =
  | {
      type: "source";
      payload: string;
    }
  | {
      type: "file";
      payload: File;
    };
</script>

<template>
  <div>
    <div class="flex mt-2 mb-4">
      <InputSelect
        v-model="lang"
        :items="langs"
        placeholder="选择语言"
        class="w-32"
      />
      <UBtn class="mx-2" @click="onSubmit">提交</UBtn>
      <div class="grow"></div>
      <RouterTabsBar
        :items="[
          {
            title: '代码',
            key: 'problem-id-submit-source',
            link: '/problem/' + $route.params.id + '/submit/source',
          },
          {
            title: '文件',
            key: 'problem-id-submit-file',
            link: '/problem/' + $route.params.id + '/submit/file',
          },
        ]"
      />
    </div>

    <NuxtPage @change="onChangeSubmission" />
  </div>
</template>
