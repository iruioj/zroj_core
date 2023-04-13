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
</script>

<template>
  <span>
    <span v-if="elapse < minute">{{ Math.round(elapse / second) }} 秒</span>
    <span v-else-if="elapse < hour">{{ Math.round(elapse / minute) }} 分钟</span>
    <span v-else-if="elapse < day">{{ Math.round(elapse / hour) }} 小时</span>
    <span v-else-if="elapse < month">{{ Math.round(elapse / day) }} 天</span>
    <span v-else>{{ Math.round(elapse / month) }} 个月</span>
  </span>
</template>