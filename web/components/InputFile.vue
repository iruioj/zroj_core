<!-- 单个文件读入 -->
<script lang="ts" setup>
const fileinput = ref<HTMLInputElement | null>(null);
const cur = ref<File | null>(null);
const onFileChange = () => {
  const files = fileinput.value?.files;
  if (files) {
    cur.value = files[0];
    emit('change', cur.value)
  }
};

const emit = defineEmits<{
  (e: "change", payload: File): void;
}>();
</script>

<template>
  <div class="flex">
    <UBtn @click="fileinput?.click()">选择文件</UBtn>
    <input
      ref="fileinput"
      type="file"
      class="hidden"
      @change="onFileChange"
    />
    <div v-if="cur" class="py-1 px-2">
      {{ cur.name }} (<SizeNum :bytes="cur.size" />) (<TimeElapse
          :elapse="Date.now() - cur.lastModified"
      />前)
    </div>
  </div>
</template>