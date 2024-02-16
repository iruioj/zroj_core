<script setup lang="ts">
const auth = useAuth();
const { info, error } = useMsgStore();

const onLogout = async () => {
  const res = await fetch(useRuntimeConfig().public.apiBase + "/auth/logout", {
    method: "POST",
    credentials: "include",
  });
  if (res.ok) {
    await auth.refresh();
    info("已退出登录");
    navigateTo("/");
    return;
  }
  error(await res.text());
};
const items = [
  [{
    slot: 'account',
	to: '/user/me'
  }], [{
    label: 'Edit Profile',
    icon: 'i-heroicons-cog-8-tooth',
	to: '/user/me/edit' 
  }], [{
    label: 'Sign out',
	click: onLogout,
    icon: 'i-heroicons-arrow-left-on-rectangle'
  }]
]
</script>

<template>
	<UDropdown :items="items" :ui="{ item: { disabled: 'cursor-text select-text' } , width: 'w-32'}" :popper="{ placement: 'bottom-end', arrow: true }">
	<div class="flex items-center">
	<UAvatar :src="'/api/user/gravatar?email='+auth.email" />
	</div>
		<template #account="{ item }">
		  <div class="text-left">
			<p class="opacity-50">
			  Signed in as
			</p>
			<p class="text-brand">
			  {{auth.username}}
			</p>
		  </div>
		</template>
	</UDropdown>
</template>

<style scoped>
</style>
