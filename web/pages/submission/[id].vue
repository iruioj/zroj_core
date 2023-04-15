<script setup lang="ts">
import { ref } from "vue";

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

const data = genSubmission(3);
const activeTask = ref([-1, -1]);
</script>

<template>
  <PageContainer>
    <div>
      <table
        class="border-collapse w-full my-2 text-sm sm:text-md border border-table"
      >
        <thead>
          <tr class="text-brand">
            <th class="border py-1 w-20">ID</th>
            <th class="border py-1 text-left px-1">Verdict</th>
            <th class="border py-1 px-1">Author</th>
            <th class="border py-1 px-1">Lang</th>
            <th class="border py-1">Time</th>
            <th class="border py-1">Memory</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td class="border py-1 w-20 text-center">#1</td>
            <td class="border py-1 text-left px-1">Wrong Answer</td>
            <td class="border py-1 px-1 text-center">{{ data.meta.author }}</td>
            <td class="border py-1 px-1 text-center">{{ data.meta.lang }}</td>
            <td class="border py-1 text-center">114</td>
            <td class="border py-1 text-center">514</td>
          </tr>
        </tbody>
      </table>

      <table
        class="border-collapse w-full my-2 text-sm sm:text-md border border-table"
      >
        <thead>
          <tr class="text-brand">
            <th class="border py-1 w-20">ID</th>
            <th class="border py-1 text-left px-1">Verdict</th>
            <th class="border py-1">Time</th>
            <th class="border py-1">Memory</th>
          </tr>
        </thead>
        <tbody v-if="data.detail.detail.Subtask">
          <template
            v-for="(subtask, sid) in data.detail.detail.Subtask"
            :key="sid"
          >
            <tr>
              <td class="border py-1 px-1 text-center">
                Subtask #{{ sid + 1 }}
              </td>
              <td class="border py-1 px-2">
                {{ subtask.status.name }}
              </td>
              <td class="border py-1 text-center">
                {{ subtask.time }}
              </td>
              <td class="border py-1 text-center">
                {{ subtask.memory }}
              </td>
            </tr>
            <template v-for="(task, tid) in subtask.tasks" :key="tid">
              <tr>
                <td
                  class="border py-1 px-1 text-center select-none cursor-pointer"
                  :rowspan="1 + 2 * task.payload.length"
                  @click="activeTask = [sid, tid]"
                >
                  Test #{{ tid + 1 }}
                </td>
                <td class="border py-1 px-2">
                  {{ task.status.name }}
                </td>
                <td class="border py-1 text-center">
                  {{ task.time }}
                </td>
                <td class="border py-1 text-center">
                  {{ task.memory }}
                </td>
              </tr>
              <template
                v-for="([title, content], id) in task.payload"
                :key="id"
              >
                <tr
                  :class="
                    !(activeTask[0] == sid && activeTask[1] == tid) &&
                    'collapse'
                  "
                >
                  <td class="text-brand font-bold px-2 border" colspan="3">
                    {{ title }}
                  </td>
                </tr>
                <tr
                  :class="[
                    !(activeTask[0] == sid && activeTask[1] == tid) &&
                      'collapse',
                  ]"
                >
                  <td colspan="3" class="border">
                    <pre class="p-2">{{ content.str }}</pre>
                  </td>
                </tr>
              </template>
            </template>
          </template>
        </tbody>
      </table>
    </div>

    <CodeBlock :raw="raw" lang="cpp" />
  </PageContainer>
</template>
