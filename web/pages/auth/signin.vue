<script setup lang="ts">
const username = ref("");
const passwd = ref("");
const msg = ref("")

const onSubmit = async (e: Event) => {
  e.preventDefault()

  try {
    if (process.client) {
      const res = await fetch(useRuntimeConfig().public.apiBase + "/auth/login", {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          username: username.value,
          passwordHash: passwd.value,
        }),
        credentials: 'include'
      })
      if (res.status == 200) {
        msg.value = '登陆成功'
        setTimeout(() => {
          navigateTo('/')
        }, 200)
      } else {
        msg.value = '登陆失败：' + await res.text()
      }
    }
  } catch (e) {
    console.log(e)
  }
}
</script>

<template>
  <form @submit="onSubmit">
    <InputText v-model="username" placeholder="用户名" />
    <InputText v-model="passwd" class="mt-1" placeholder="密码" type="password" autocomplete="current-password" />
    <div class="mt-1">
      <UBtn class="w-full">提交</UBtn>
    </div>
  </form>
</template>
