<script setup lang="ts">
const props = defineProps<{
  title?: string;
  // default value of title. use it to control expanding
  id?: string;
  meta: TaskMeta;
}>();

const cur = useSubmExpandID();

const onToggle = () => {
  const id = props.id || props.title || "";
  if (cur.value == id) {
    cur.value = id.split(".").slice(0, -1).join(".");
  } else {
    cur.value = id;
  }
};
</script>

<template>
  <div
    class="flex"
    :class="id && 'cursor-pointer'"
    :data-expand-id="id || title || ''"
    @click="onToggle"
  >
    <div v-if="title" class="p-2 font-bold">{{ title }}</div>
    <div class="p-2">{{ statusTitle[meta.status.name] }}</div>
    <div class="p-2">{{ Math.round(meta.score_rate * 100) }}pts</div>
    <div class="p-2">Time: {{ meta.time }}ms</div>
    <div class="p-2">Memory: {{ (meta.memory / 1e6).toFixed(3) }}MB</div>
  </div>
</template>
