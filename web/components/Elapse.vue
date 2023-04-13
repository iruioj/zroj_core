<script setup lang="ts">
import { h } from 'nuxt/dist/app/compat/capi'

const props = defineProps<{
  elapse: number
}>()

const second = 1000
const minute = 60 * second
const hour = 60 * minute
const day = 24 * hour
const month = day * 30

const data = computed(() => {
  let x = props.elapse
  return {
    month: Math.floor(x / month),
    day: Math.floor(x % month / day),
    hour: Math.floor(x % day / hour),
    minute: Math.floor(x % hour / minute),
    second: Math.floor(x % minute / second),
  }
})
</script>

<template>
  <span>
    <span class="pl-1" v-if="data.month">{{ data.month }} 个月</span>
    <span class="pl-1" v-if="data.day">{{ data.day }} 天</span>
    <span class="pl-1" v-if="data.hour">{{ data.hour }} 小时</span>
    <span class="pl-1" v-if="data.minute">{{ data.minute }} 分钟</span>
    <span class="pl-1" v-if="data.second">{{ data.second }} 秒</span>
  </span>
</template>