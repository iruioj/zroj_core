<script setup lang="ts">
import { probId } from '../[id].vue';

const { error, info } = useMsgStore();

const file = ref<File| null>(null)
const id = inject(probId, null)

const onChange = (f: File) => {
    file.value = f
}

const onSubmit = async (e: Event) => {
  e.preventDefault();
  console.log('submit')
  let formdata = new FormData()
  if (!id) {
    throw 'no problem id'
  }
  if (!file.value) {
    throw 'no files'
  }
  formdata.append('id', id.value.toString())
  formdata.append('data', file.value)
  let r = await useAPI().problem.fulldata.post(formdata)
  console.log(r.data.value)
  
  info('上传成功')
}
</script>


<template>
  <div>
    <div class="my-1">上传题目文件</div>
    <InputFile class="my-1" @change="onChange" />
    <UBtn class="my-1" @click="onSubmit">提交</UBtn>
  </div>
</template>