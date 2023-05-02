<script setup lang="ts">
import { genContests } from "~/utils/gen_contests";

const data = {
  page: {
    cur: 2,
    totalPage: 26,
    isFirst: false,
    isLast: false,
  },
  contests: genContests(10),
};
</script>

<template>
  <PageContainer>
    <SectionContainer title="比赛列表">
      <div class="py-2">
        <table class="w-full text-sm hidden sm:table">
          <thead>
            <TableHeaderRow>
              <th class="px-2 pb-2 text-left">比赛</th>
              <th class="pb-2 text-center">开始时间/时长</th>
              <th class="pb-2">报名人数</th>
            </TableHeaderRow>
          </thead>
          <tbody>
            <TableRow v-for="p in data.contests" :key="p.id">
              <td class="p-2">
                <TextLink :to="p.link">{{ p.title }}</TextLink>
              </td>
              <td class="py-2 text-center flex flex-col whitespace-nowrap">
                <DateTime :time="p.startTime" />
                <TimeElapse :elapse="p.duration" />
              </td>
              <td class="py-2 text-center">
                {{ p.participants }}
              </td>
            </TableRow>
          </tbody>
        </table>

        <ul class="sm:hidden">
          <li
            v-for="p in data.contests"
            :key="p.id"
            class="py-2 border-b border-theme"
          >
            <div class="text-md pb-1">
              <TextLink :to="p.link">{{ p.title }}</TextLink>
            </div>
            <div>
              <NuxtIcon name="schedule" class="inline-block align-middle" />{{
                " "
              }}
              <DateTime :time="p.startTime" />
            </div>
            <div>
              <NuxtIcon name="timer" class="inline-block align-middle" />
              <TimeElapse :elapse="p.duration" />
            </div>
            <div>
              <NuxtIcon name="group" class="inline-block align-middle" />
              {{ p.participants }}
            </div>
          </li>
        </ul>

        <div class="flex justify-center">
          <div v-if="!data.page.isFirst"><button>上一页</button></div>
          <div class="mx-4">
            第 {{ data.page.cur }} 页 / 共 {{ data.page.totalPage }} 页
          </div>
          <div v-if="!data.page.isLast"><button>下一页</button></div>
        </div>
      </div>
    </SectionContainer>
  </PageContainer>
</template>
