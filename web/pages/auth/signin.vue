<script setup lang="ts">
const username = ref("");
const passwd = ref("");
</script>

<template>
  <form>
    <InputText v-model="username" placeholder="用户名" />
    <InputText
      v-model="passwd"
      class="mt-1"
      placeholder="密码"
      type="password"
      autocomplete="current-password"
    />
    <div class="mt-1">
      <UBtn class="w-full">提交</UBtn>
    </div>
  </form>
</template>
