import { defineStore } from "pinia";

// todo: https://github.com/damien-hl/nuxt3-auth-example/blob/main/composables/auth/useAuth.ts
export const useAuth = defineStore("auth_store", () => {
  const username = ref<null | string>(null);
  const email = ref<null | string>(null);

  async function refresh() {
    if (process.client) {
      const res = await fetch(
        useRuntimeConfig().public.apiBase + "/auth/info",
        {
          method: "GET",
          headers: {
            "Content-Type": "application/json",
          },
          mode: "cors",
        },
      );
      if (res.ok) {
        const data = await res.json();
        console.log(data.email);
        username.value = data.username;
        email.value = data.email;
      } else {
        username.value = null;
      }
    }
  }

  if (process.client) {
    refresh();
  }

  return {
    username,
    email,
    refresh,
  };
});

type Message = {
  id: number;
  kind: "info" | "error";
  msg: string;
};

const showDuration = 1500;

export const useMsgStore = defineStore("message_list", () => {
  const count = ref(0);
  const list = ref([] as Message[]);

  function addmsg(msg: Message) {
    list.value.push(msg);
    setTimeout(() => {
      list.value.shift();
    }, showDuration);
  }
  function info(msg: string) {
    addmsg({ id: ++count.value, kind: "info", msg });
  }
  function error(msg: string) {
    addmsg({ id: ++count.value, kind: "error", msg });
  }

  return { info, error, list };
});

export const useSubmExpandID = () => useState("subm_expand_id", () => "");
