import { defineStore } from "pinia";

export const useAuth = () =>
  useLazyAsyncData("authinfo", async () => {
    if (process.server) {
      return;
    }
    const res = await fetch(useRuntimeConfig().public.apiBase + "/auth/info");
    if (res.ok) {
      return await res.json();
    }
    console.log(res.status, res);
  });

type Message = {
  id: number;
  kind: "info" | "error";
  msg: string;
};

const show_duration = 1500;

export const useMsgStore = defineStore("message_list", () => {
  const count = ref(0);
  const list = ref([] as Message[]);

  function addmsg(msg: Message) {
    list.value.push(msg);
    setTimeout(() => {
      list.value.shift();
    }, show_duration);
  }
  function info(msg: string) {
    addmsg({ id: ++count.value, kind: "info", msg });
  }
  function error(msg: string) {
    addmsg({ id: ++count.value, kind: "error", msg });
  }

  return { info, error, list };
});