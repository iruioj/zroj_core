import {
  AsyncDataExecuteOptions,
  AsyncDataRequestStatus,
} from "nuxt/dist/app/composables/asyncData";
import type { FetchError } from "ofetch";

// auto watch args if it is ref or reactive
export function callAPI(
  method: string,
  path: string,
  args?: any,
): Promise<any> {
  if (process.client) {
    console.log("client call api", method, path, args);
  }
  path = useRuntimeConfig().public.apiBase + path;

  const key = method + ":" + path;
  const fetching = useState(key + ":fetching", () => false);

  const options = {
    server: false, // 这只会降低首次加载的体验
    key,
    method: method as any,
    credentials: "include" as any,
    headers: useRequestHeaders(),
    watch: isRef(args) || isReactive(args) ? [args] : undefined,
    onRequest() {
      fetching.value = true;
    },
    onRequestError() {
      fetching.value = false;
    },
    onResponse() {
      fetching.value = false;
    },
    onResponseError() {
      fetching.value = false;
    },
  };
  if (args === undefined) {
    return useFetch(path, options).then((r) => ({ ...r, fetching }));
  } else if (method === "get") {
    return useFetch(path, { ...options, query: args }).then((r) => ({
      ...r,
      fetching,
    }));
  } else if (
    args instanceof FormData ||
    (isRef(args) && args.value instanceof FormData)
  ) {
    return useFetch(path, { ...options, body: args }).then((r) => ({
      ...r,
      fetching,
    }));
  } else {
    return useFetch(path, { ...options, body: args }).then((r) => ({
      ...r,
      fetching,
    }));
  }
}
export function fetchAPI(
  method: string,
  path: string,
  args?: any,
): Promise<any> {
  if (process.client) {
    console.log("client call api", method, path, args);
  }
  path = useRuntimeConfig().public.apiBase + path;

  const options = {
    method: method as any,
    credentials: "include" as any,
    headers: useRequestHeaders(),
  };
  if (args === undefined) {
    return $fetch(path, options);
  } else if (method === "get") {
    return $fetch(path, { ...options, query: args });
  } else if (args instanceof FormData) {
    return $fetch(path, { ...options, body: args });
  } else {
    return $fetch(path, { ...options, body: args });
  }
}

export interface ExtAsyncData<T> {
  data: Ref<T>;
  pending: Ref<boolean>;
  refresh: (opts?: AsyncDataExecuteOptions) => Promise<void>;
  execute: (opts?: AsyncDataExecuteOptions) => Promise<void>;
  error: Ref<FetchError | null>;
  status: Ref<AsyncDataRequestStatus>;
  fetching: Ref<boolean>;
}
