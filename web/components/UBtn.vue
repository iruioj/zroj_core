<script setup lang="ts">
const staticClass =
  "px-2 py-1 border rounded border-slate-400 ring-brand transition-all ring-offset-1 select-none";
const enableClass =
  "text-secondary hover:text-brand hover:border-brand focus:ring-1";
const disableClass = "text-slate-400";

const props = defineProps<{
  disable?: boolean;
}>();

const onClick = (event: Event) => {
  if (props.disable) {
    event.stopImmediatePropagation();
    event.stopPropagation();
    event.preventDefault();
  }
};
</script>

<template>
  <button
    :class="[staticClass, disable ? disableClass : enableClass]"
    @click="onClick"
  >
    <slot></slot>
  </button>
</template>
