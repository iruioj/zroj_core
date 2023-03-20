<script setup lang="ts">
import NavButton from './NavButton.vue'
import {
  HomeIcon,
  AboutIcon,
  ListIcon,
  QuizIcon,
  AssignmentIcon,
} from './icons'
</script>

<template>
  <header class="
  sticky top-0 z-30 bg-white
  flex items-center border-b border-slate-900/10 dark:border-slate-50/[0.06]">
    <div class="inline-block px-4 font-bold text-xl text-brand">ZROJ</div>
    <NavButton to="/" :icon="HomeIcon">Home</NavButton>
    <NavButton to="/problem/1" :icon="ListIcon">Problems</NavButton>
    <!-- <NavButton to="/" :icon="QuizIcon">Contests</NavButton> -->
    <NavButton to="/submission/2" :icon="AssignmentIcon">Submissions</NavButton>
    <!-- <NavButton to="/about" :icon="AboutIcon">About</NavButton> -->
  </header>
</template>
