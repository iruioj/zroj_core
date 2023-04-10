<script lang="ts" setup>
defineProps<{
  items: {
    title: string;
    key: string;
    link: string;
  }[];
  activeKey: string;
}>();
</script>

<template>
  <ul class="flex">
    <li v-for="item in items" :key="item.key">
      <NuxtLink
        :to="item.link"
        :class="[
          'px-2 underline hover:decoration-brand underline-offset-8 transition-colors cursor-pointer hover:text-brand',
          activeKey === item.key
            ? 'decoration-brand text-brand'
            : 'decoration-brand/0 text-slate-700',
        ]"
        >{{ item.title }}
      </NuxtLink>
    </li>
  </ul>
</template>
