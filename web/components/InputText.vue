<script setup lang="ts">

defineProps<{
  modelValue: string
  placeholder?: string
  type?: string,
  autocomplete?: string
}>()
defineEmits<{
  (e: 'update:modelValue', content: string): void
}>()
</script>

<template>
  <div class="border rounded py-1 px-2 border-slate-400 hover:border-brand">
    <input 
    :value="modelValue" 
      @input="$emit('update:modelValue', ($event.target as any).value)"
      class="outline-none w-full" 
      :placeholder="placeholder || ''" 
      :type="type || 'text'"
      :autocomplete="autocomplete || 'new-password'" 
      />
  </div>
</template>