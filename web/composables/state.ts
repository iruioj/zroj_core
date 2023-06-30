import { defineStore } from "pinia";

// todo: https://github.com/damien-hl/nuxt3-auth-example/blob/main/composables/auth/useAuth.ts
export const useAuth = () => useAPI().auth.info.get(); //useAPI("get:/auth/info")

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
