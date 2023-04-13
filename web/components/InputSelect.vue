<script setup lang="ts">
import { ExpandMoreIcon } from "~/components/icons";

type Item = {
  title: string;
  value: string;
};

const props = defineProps<{
  items: Item[];
  placeholder?: string;
}>();

const emit = defineEmits<{
  (e: "change", payload: Item): void;
}>();

const showSelect = ref(false);
const selected = ref<{ title: string; value: string } | null>(null);
const onSelect = (value: string) => {
  const item = props.items.find((o) => o.value === value);
  if (item) {
    selected.value = item;
    emit("change", item);
  }
  showSelect.value = false;
};
</script>

<template>
  <div class="group relative">
    <div
      class="flex pl-2 border rounded border-slate-400 group-hover:border-brand cursor-pointer text-slate-500"
      @click="showSelect = !showSelect"
    >
      <div class="w-full select-none py-1" :class="!selected && 'text-secondary'">
        {{ selected?.title || placeholder || "" }}
      </div>
      <ExpandMoreIcon
        class="bg-white/0 h-8 w-7 p-1 fill-secondary group-hover:fill-brand transition-transform"
        :class="showSelect && 'rotate-180'"
      />
    </div>
    <div v-if="showSelect" class="absolute w-full">
      <ul class="bg-back border border-black/10 shadow-xl w-full">
        <li
          v-for="item in items"
          :key="item.value"
          class="p-2 select-none hover:text-brand cursor-pointer border-b border-dashed border-b-black/20 last:border-b-0"
          :class="selected?.value === item.value && 'text-brand'"
          @click="onSelect(item.value)"
        >
          {{ item.title }}
        </li>
      </ul>
    </div>
  </div>
</template>
