<script setup lang="ts">

const username = ref("");
const passwd = ref("");
const email = ref("");
const msg = ref("")

const onSubmit = async (e: Event) => {
  e.preventDefault()

  try {
    const res = await fetch(useRuntimeConfig().public.apiBase + "/auth/register", {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      mode: 'cors',
      body: JSON.stringify({
        email: email.value,
        username: username.value,
        passwordHash: passwd.value,
      })
    })
    if (res.status == 200) {
      msg.value = '注册成功，跳转到登陆...'
      setTimeout(() => {
        navigateTo('/auth/signin')
      }, 300)
    } else {
      msg.value = '注册失败：' + await res.text()
    }
  } catch (e) {
    console.log(e)
  }
}
</script>

<template>
  <form @submit="onSubmit">
    <InputText v-model="email" placeholder="邮箱" />
    <InputText v-model="username" class="mt-1" placeholder="用户名" />
    <InputText v-model="passwd" class="mt-1" placeholder="密码" type="password" />
    <div class="mt-1">
      <UBtn class="w-full">提交</UBtn>
    </div>
    <div class="mt-1">
      <span class="text-brand">{{ msg }}</span>
    </div>
  </form>
</template>
