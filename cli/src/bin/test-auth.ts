import {
  CookieJar,
  wrapFetch,
} from "https://deno.land/x/another_cookiejar@v5.0.3/mod.ts";

// you can also pass your own cookiejar to wrapFetch to save/load your cookies
const cookieJar = new CookieJar();
// Now use this fetch and any cookie that is set will be sent with your next requests automatically
const fetch = wrapFetch({ cookieJar })

const registerRes = await fetch("http://127.0.0.1:8080/auth/register", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
  },
  body: JSON.stringify({
    email: "test@test.com",
    username: "testit",
    passwordHash: "hash",
  }),
});

// console.log(registerRes);
console.log(await registerRes.text());

const loginRes = await fetch("http://127.0.0.1:8080/auth/login", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
  },
  body: JSON.stringify({
    username: "testit",
    passwordHash: "hash",
  }),
});

// console.log(loginRes, loginRes.headers.get("set-cookie"));
console.log(await loginRes.text());
console.log(cookieJar.cookies)

// 两次 inspect 返回的结果应当一样，说明鉴权是有效的
const inspectRes = await fetch("http://127.0.0.1:8080/auth/inspect", {
  method: "GET",
});
// console.log(inspectRes);
console.log(await inspectRes.json());
const inspectRes2 = await fetch("http://127.0.0.1:8080/auth/inspect", {
  method: "GET",
});
console.log(await inspectRes2.json());
