<script setup lang="ts">
const props = defineProps<{
  pid: number;
}>();
const { error, info } = useMsgStore();

const file = ref<File | null>(null);
const { data: fulldataMeta } = await useAPI().problem.fulldata_meta.get.use({
  id: props.pid,
});

const onChange = (f: File) => {
  file.value = f;
};

const onSubmit = async (e: Event) => {
  e.preventDefault();
  console.log("submit");
  const formdata = new FormData();
  if (!file.value) {
    throw "no files";
  }
  formdata.append("id", props.pid.toString());
  formdata.append("data", file.value);
  const r = await useAPI().problem.fulldata.post.use(formdata);
  console.log(r.data.value);

  info("上传成功");
};
</script>

<template>
  <div>
    <CodeBlock v-if="fulldataMeta" :raw="String(fulldataMeta)" lang="" />
    <div v-else>暂无数据</div>
    <div class="my-1">上传题目文件</div>
    <InputFile class="my-1" @change="onChange" />
    <UBtn class="my-1" @click="onSubmit">提交</UBtn>
  </div>
</template>
