<!-- 使用 aleph 指示 hydration -->
<script setup lang="ts">
const auth = useAuth();
const { list } = useMsgStore();


</script>

<template>
  <header class="bg-back fixed w-full top-0 z-30 border-b border-theme">
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
        <NavButton to="/submissions">Submissions</NavButton>
        <NavButton to="/oneoff">Customtest</NavButton>
      </div>
      <div class="grow"></div>

		<AvatarDropdown v-if="auth.username" :username="auth.username" class="px-4 print:hidden"/>
        <TextLink v-else to="/auth/signin" class="px-4 py-2 print:hidden">Sign In/Up</TextLink>
    </div>
    <div class="text-xs mx-2 print:hidden sm:hidden border-t border-theme">
      <NavButton to="/problemset">Problems</NavButton>
      <NavButton to="/contests">Contests</NavButton>
      <NavButton to="/submissions">Submissions</NavButton>
      <NavButton to="/oneoff">Customtest</NavButton>
    </div>
    <TransitionGroup name="msg-list" tag="div">
      <div
        v-for="msg in list"
        :key="msg.id"
        class="p-1 text-center"
        :class="'msg-' + msg.kind"
      >
        <NuxtIcon
          v-if="msg.kind == 'error'"
          class="inline-block align-middle"
          name="error"
        />
        {{ msg.msg }}
      </div>
    </TransitionGroup>
  </header>
</template>
<style>
.msg-list-move,
.msg-list-enter-active,
.msg-list-leave-active {
  transition: all 0.5s ease;
}

.msg-list-enter-from,
.msg-list-leave-to {
  opacity: 0;
}

/* ensure leaving items are taken out of layout flow so that moving
   animations can be calculated correctly. */
.msg-list-leave-active {
  position: absolute;
  width: 100%;
}
</style>
