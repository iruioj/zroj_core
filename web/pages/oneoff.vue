<script lang="ts" setup>
const sourceRef = ref<HTMLElement | null>(null);
const code = ref("");
const autoResize = () => {
  const e = sourceRef.value;
  if (e) {
    e.style.height = e.scrollHeight + "px";
  }
};
const submitCode = () => {
  console.log(code.value);
};
// https://coolors.co/gradient-maker/c70000-ec6600
</script>

<template>
  <div class="w-[700px] m-auto">
    <div class="mt-8 mb-4 text-4xl text-brand font-medium">自定义测试</div>
    <textarea
      ref="sourceRef"
      v-model="code"
      class="font-mono w-full overflow-hidden border border-slate-400 rounded outline-none focus:ring-2 focus:ring-brand focus:ring-offset-2"
      @input="autoResize"
    ></textarea>

    <button
      class="focus:ring-offset-2 focus:ring-2 focus:ring-brand rounded border border-brand px-3 py-1.5 text-brand hover:bg-brand hover:text-white ease-in-out duration-200 transition"
      @click="submitCode"
    >
      提交
    </button>
  </div>
</template>
