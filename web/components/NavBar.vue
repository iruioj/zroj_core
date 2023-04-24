<!-- 使用 aleph 指示 hydration -->
<script setup lang="ts">
import NavButton from "./NavButton.vue";

const username = useUsername(); // useCookie("username")
</script>

<template>
  <header
    class="bg-back sticky top-0 z-30 border-b border-slate-900/10 dark:border-slate-50/[0.06]"
  >
    <div class="flex items-center">
      <NuxtLink to="/">
        <div class="inline-block px-4 font-bold text-xl text-brand">
          ZROJ
          <ClientOnly>
            <sup class="text-brand" title="hydration complete">ℵ</sup>
          </ClientOnly>
        </div>
      </NuxtLink>
      <div class="print:hidden hidden sm:flex">
        <NavButton to="/problemset">Problems</NavButton>
        <NavButton to="/contests">Contests</NavButton>
        <NavButton to="/submission/2">Submissions</NavButton>
        <NavButton to="/oneoff">Customtest</NavButton>
      </div>
      <div class="grow"></div>
      <div>
        <div class="py-2 px-4 print:hidden">
          <TextLink v-if="username" to="/user/me">{{ username }}</TextLink>
          <TextLink v-else to="/auth/signin">Sign In/Up</TextLink>
        </div>
      </div>
    </div>
    <div
      class="text-xs mx-2 print:hidden sm:hidden border-t border-slate-900/10 dark:border-slate-50/[0.06]"
    >
      <NavButton to="/problemset">Problems</NavButton>
      <NavButton to="/contests">Contests</NavButton>
      <NavButton to="/submission/2">Submissions</NavButton>
      <NavButton to="/oneoff">Customtest</NavButton>
    </div>
  </header>
</template>
