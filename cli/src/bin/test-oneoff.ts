import {
  CookieJar,
  wrapFetch,
} from "https://deno.land/x/another_cookiejar@v5.0.3/mod.ts";

// you can also pass your own cookiejar to wrapFetch to save/load your cookies
const cookieJar = new CookieJar();
// Now use this fetch and any cookie that is set will be sent with your next requests automatically
const fetch = wrapFetch({ cookieJar });

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
console.log(registerRes.status, await registerRes.text());

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
console.log(registerRes.status, await loginRes.text());

const inspectRes = await fetch("http://127.0.0.1:8080/auth/inspect", {
  method: "GET",
});
console.log(inspectRes.status, await inspectRes.json());

const data = new FormData();
const src = `#include<bits/stdc++.h>
using namespace std;

int main() {
  int a, b;
  cin >> a >> b;
  cout << a + b << endl;
  return 0;
}
`;
const srcFile = new File([src], "main.gnu_cpp14_o2.cpp");

data.append("source", srcFile);
data.append("input", `1 2`, "input.txt");

const submitRes = await fetch("http://127.0.0.1:8080/custom_test", {
  method: "POST",
  body: data,
});

console.log(submitRes.status, await submitRes.text());

const handler = setInterval(async () => {
  const queryRes = await fetch("http://127.0.0.1:8080/custom_test", {
    method: "GET",
  });
  const data = await queryRes.json()
  console.log(queryRes.status, data);
  if (data.result != null) {
    clearInterval(handler)
  }
}, 500);
