<script setup lang="ts">
// import CodeEditor from '@/components/CodeEditor.vue'
import { ref } from "vue";
import {
  DoneIcon,
  PersonIcon,
  CodeIcon,
  GlobeAsiaIcon,
  TimerIcon,
  MemoryIcon,
  ExpandMoreIcon,
} from "../../components/icons";

const raw = ref(`#include <bits/stdc++.h>
#define FOR(a, b, c) for (int a = (int)(b); a <= (int)(c); a++)
#define ROF(a, b, c) for (int a = (int)(b); a >= (int)(c); a--)
using namespace std;

struct Edge {
  int u, v, w, ow;
  Edge(int _u, int _v, int _w) { u = _u, v = _v, w = ow = _w; }
  void reset() { w = ow; }
};

/**
 * Chu-Liu/Edmonds' algorithm
 * 计算有向图（允许重边、不允许自环）给定根的最小权外向生成树（最小树形图）
 * vector<Edge> buildFrom(n, r, ve): n 个点，边集是 ve，根是 r 的最小权外向生成树
 *   若无解则返回一个空的 vector
 *   要求 ve 非空
 */
template <const int N, const int M> struct DirectedMST {
  int nd[N], tnd[N], fa[N], pre[N], In[N], Time[M], totTime, onCir[N], totCir;
  vector<int> toggle[M];

  int get(int u) { return fa[u] == u ? u : fa[u] = get(fa[u]); }
  int getNode(int u) { return nd[u] == u ? u : nd[u] = getNode(nd[u]); }

  bool work(const int n, const int root, vector<Edge> &ve) {
    bool flag = false;
    fill(In, In + n + 1, -1);
    fill(onCir, onCir + n + 1, 0);
    totCir = 0;

    for (unsigned i = 0; i < ve.size(); i++) {
      int u = getNode(ve[i].u), v = getNode(ve[i].v);
      if (u == v) continue;
      if (In[v] == -1 || ve[In[v]].w > ve[i].w) In[v] = i;
    }

    FOR(i, 1, n) fa[i] = i;

    FOR(i, 1, n) if (i != root && getNode(i) == i) {
      if (In[i] == -1) return false;
      Edge e = ve[In[i]];
      int u = getNode(e.u), v = getNode(e.v);
      if (u == v) continue;
      if (get(u) == get(v)) {
        ++totCir;
        for (int z = u; z != -1; z = z == v ? -1 : getNode(ve[In[z]].u))
          onCir[z] = totCir, tnd[z] = v, Time[In[z]] = ++totTime; // assert(z);
        flag = true;
      } else {
        fa[get(u)] = get(v);
      }
    }

    for (unsigned i = 0; i < ve.size(); i++) {
      auto &e = ve[i];
      int u = getNode(e.u), v = getNode(e.v);
      if (u == v) continue;
      if (onCir[v] && onCir[v] == onCir[u]) continue;
      if (onCir[v]) toggle[i].push_back(In[v]), e.w -= ve[In[v]].w;
    }

    FOR(i, 1, n) if (onCir[i]) nd[i] = tnd[i]; // assert(getNode(i) == i);

    return flag;
  }
  vector<Edge> buildFrom(int n, int root, vector<Edge> ve) {
    assert(!ve.empty());
    vector<Edge> vt;
    FOR(i, 1, n) nd[i] = i;
    fill(Time, Time + ve.size() + 1, 0);
    totTime = 0;

    while (work(n, root, ve))
      ;

    FOR(i, 1, n) if (getNode(i) == i && i != root) {
      if (In[i] == -1) return vt; // empty
      Time[In[i]] = ++totTime;
    }
    vector<int> SortByTime(totTime + 1, -1);
    for (unsigned i = 0; i < ve.size(); i++)
      if (Time[i]) SortByTime[Time[i]] = i;

    ROF(i, totTime, 1) {
      int x = SortByTime[i];
      if (Time[x])
        for (int y : toggle[x]) Time[y] = 0;
    }

    for (unsigned i = 0; i < ve.size(); i++) {
      ve[i].reset();
      if (Time[i]) vt.push_back(ve[i]);
    }
    assert(vt.size() == n - 1);
    return vt;
  }
};
`);

const data = {
  type: "subtasks",
  subtasks: [
    {
      verdict: "Accepted",
      tests: [
        {
          verdict: "Accepted",
          time: 114,
          memory: 514,
        },
        {
          verdict: "Accepted",
          time: 114,
          memory: 514,
        },
        {
          verdict: "Accepted",
          time: 114,
          memory: 514,
        },
      ],
    },
    {
      verdict: "Accepted",
      tests: [
        {
          verdict: "Accepted",
          time: 114,
          memory: 514,
        },
        {
          verdict: "Accepted",
          time: 114,
          memory: 514,
        },
      ],
    },
    {
      verdict: "Time Limit Exceeded",
      tests: [
        {
          verdict: "Time Limit Exceeded",
          time: null,
          memory: 514,
        },
        {
          verdict: "Skipped",
          time: null,
          memory: null,
        },
      ],
    },
    {
      verdict: "Skipped",
      tests: [
        { verdict: "Skipped", time: null, memory: null },
        { verdict: "Skipped", time: null, memory: null },
        { verdict: "Skipped", time: null, memory: null },
      ],
    },
  ],
};
</script>

<template>
  <div>
    <div class="w-[700px] m-auto">
      <div class="meta grid grid-cols-3 gap-1 my-1">
        <AppBadge :icon="DoneIcon">Accepted</AppBadge>
        <AppBadge :icon="PersonIcon">Sshwy</AppBadge>
        <AppBadge :icon="CodeIcon">1.2kb</AppBadge>
        <AppBadge :icon="GlobeAsiaIcon">C/C++</AppBadge>
        <AppBadge :icon="TimerIcon">114ms</AppBadge>
        <AppBadge :icon="MemoryIcon">514ms</AppBadge>
      </div>

      <div>
        <template v-for="(subtask, index) in data.subtasks" :key="index">
          <div
            class="px-2 py-1 mb-1 rounded border cursor-pointer border-red-800 flex"
          >
            <div>Subtask #{{ index + 1 }}</div>
            <div class="flex mx-2 py-1">
              <template v-for="(test, j) in subtask.tests" :key="j">
                <div
                  :class="[
                    'mr-1 w-4 h-4 border-2 rounded border-red-800',
                    test.verdict === 'Accepted' ? 'rounded-full' : '',
                    test.verdict === 'Skipped'
                      ? 'rounded-full border-dotted'
                      : '',
                  ]"
                ></div>
              </template>
            </div>
            <div class="grow"></div>
            <ExpandMoreIcon fill="#991b1b" class="w-6 h-6" />
          </div>
          <div
            v-for="(test, j) in subtask.tests"
            :key="j"
            class="px-2 py-1 mb-1 mx-2 rounded border cursor-pointer border-red-800"
          >
            Testcase #{{ j + 1 }} Verdict: {{ test.verdict }}
          </div>
        </template>
      </div>

      <CodeBlock :raw="raw" lang="cpp" />
    </div>
  </div>
</template>
