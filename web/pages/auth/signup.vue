<script setup lang="ts">
const username = ref("");
const passwd = ref("");
const email = ref("");
</script>

<template>
  <form>
    <InputText v-model="email" placeholder="邮箱" />
    <InputText v-model="username" class="mt-1" placeholder="用户名" />
    <InputText
      v-model="passwd"
      class="mt-1"
      placeholder="密码"
      type="password"
    />
    <div class="mt-1">
      <UBtn class="w-full">提交</UBtn>
    </div>
  </form>
</template>
