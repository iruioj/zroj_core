<script setup lang="ts">
const username = ref(''), passwd = ref(''), email = ref('')
</script>

<template>
    <form>
      <InputText v-model="email" placeholder="邮箱" />
      <InputText class="mt-1" v-model="username" placeholder="用户名" />
      <InputText class="mt-1" v-model="passwd" placeholder="密码" type="password" />
      <div class="mt-1">
        <UBtn class="w-full">提交</UBtn>
      </div>
    </form>
</template>