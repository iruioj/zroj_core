<script setup lang="ts">
import NavButton from './NavButton.vue'

</script>

<template>
  <header class="sticky top-0 z-30 bg-white
        flex items-center 
        border-b border-slate-900/10 dark:border-slate-50/[0.06]">
    <NuxtLink to="/">
      <div class="inline-block px-4 font-bold text-xl text-brand">ZROJ</div>
    </NuxtLink>
    <div class="flex">
      <!-- <NavButton to="/" :icon="HomeIcon">Home</NavButton> -->
      <NavButton to="/problem/1">Problems</NavButton>
      <!-- <NavButton to="/" :icon="QuizIcon">Contests</NavButton> -->
      <NavButton to="/submission/2">Submissions</NavButton>
      <NavButton to="/oneoff">Custom Test</NavButton>
    </div>
  </header>
</template>
