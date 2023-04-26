<script setup lang="ts">
const props = defineProps<{
  elapse: number;
}>();

const second = 1000;
const minute = 60 * second;
const hour = 60 * minute;
const day = 24 * hour;
const month = day * 30;

const data = computed(() => {
  const x = props.elapse;
  return {
    month: Math.floor(x / month),
    day: Math.floor((x % month) / day),
    hour: Math.floor((x % day) / hour),
    minute: Math.floor((x % hour) / minute),
    second: Math.floor((x % minute) / second),
  };
});
</script>

<template>
  <span>
    <span v-if="data.month" class="pl-1">{{ data.month }} 个月</span>
    <span v-if="data.day" class="pl-1">{{ data.day }} 天</span>
    <span v-if="data.hour" class="pl-1">{{ data.hour }} 小时</span>
    <span v-if="data.minute" class="pl-1">{{ data.minute }} 分钟</span>
    <span v-if="data.second" class="pl-1">{{ data.second }} 秒</span>
  </span>
</template>
