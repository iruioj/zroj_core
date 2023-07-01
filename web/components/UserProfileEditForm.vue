<script setup lang="ts">
const { error } = useMsgStore();
const props = defineProps<{
  initdata: {
    id: number;
    username: string;
    email: string;
    motto: string;
    name: string;
    register_time: string;
    gender: "Male" | "Female" | "Others" | "Private";
  };
}>();

const data = reactive(props.initdata);

const onSubmit = async (e: Event) => {
  e.preventDefault();
  console.log("submit");
  try {
    await useAPI().user.edit.post({
      // password_hash: undefined,
      // gender: undefined,
      // email: undefined,
      motto: data.motto,
      name: data.name,
    });
    navigateTo("/user/me");
  } catch (e) {
    error((e as any).message);
  }
};
</script>
<template>
  <form @submit="onSubmit">
    <table class="w-full">
      <!-- for styling -->
      <tbody>
        <tr>
          <td class="w-20 text-right pr-1 my-1">用户名</td>
          <td class="py-1">
            <InputText v-model="data.username" readonly />
          </td>
        </tr>
        <tr>
          <td class="text-right pr-1">邮箱</td>
          <td class="py-1">
            <InputText v-model="data.email" readonly />
          </td>
        </tr>
        <tr>
          <td class="text-right pr-1">注册时间</td>
          <td class="py-1">
            <InputText v-model="data.register_time" readonly />
          </td>
        </tr>
        <tr>
          <td class="text-right pr-1">姓名</td>
          <td class="py-1">
            <InputText v-model="data.name" />
          </td>
        </tr>
        <tr>
          <td class="text-right pr-1">格言</td>
          <td class="py-1">
            <InputText v-model="data.motto" />
          </td>
        </tr>
      </tbody>
    </table>
    <div class="mt-1">
      <UBtn class="w-full">提交</UBtn>
    </div>
  </form>
</template>
