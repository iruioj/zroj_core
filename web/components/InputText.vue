<script setup lang="ts">
defineProps<{
  modelValue: string;
  placeholder?: string;
  type?: string;
  autocomplete?: string;
  readonly?: boolean; // default: false
}>();
const emit = defineEmits<{
  (e: "update:modelValue", content: string): void;
}>();

const onInput = (event: Event) => {
  // event.preventDefault()
  // console.log('input', event)
  // if (props.readonly) return
  // console.log('update')
  emit("update:modelValue", (event.target as any).value);
};
</script>

<template>
  <div
    :class="[
      'border rounded py-1 px-2 border-slate-400',
      !readonly && 'hover:border-brand',
    ]"
  >
    <input
      :value="modelValue"
      class="outline-none w-full"
      :placeholder="placeholder || ''"
      :type="type || 'text'"
      :autocomplete="autocomplete || 'new-password'"
      :readonly="readonly"
      @input="onInput"
    />
  </div>
</template>
