<script setup lang="ts">
import NavButton from "./NavButton.vue";
</script>

<template>
  <header
    class="sticky top-0 z-30 bg-white flex items-center border-b border-slate-900/10 dark:border-slate-50/[0.06]"
  >
    <NuxtLink to="/">
      <div class="inline-block px-4 font-bold text-xl text-brand">ZROJ</div>
    </NuxtLink>
    <div class="flex">
      <NavButton to="/problemset">Problems</NavButton>
      <!-- <NavButton to="/" :icon="QuizIcon">Contests</NavButton> -->
      <NavButton to="/submission/2">Submissions</NavButton>
      <NavButton to="/oneoff">Custom Test</NavButton>
    </div>
    <div class="grow"></div>
    <div>
      <div class="py-2 px-4">
        <TextLink to="/auth/signin">Sign In/Up</TextLink>
      </div>
    </div>
  </header>
</template>
