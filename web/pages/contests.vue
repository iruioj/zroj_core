<script setup lang="ts">
import { genContest, genContests } from '~/utils/gen_contests';

const data = {
  page: {
    cur: 2,
    totalPage: 26,
    isFirst: false,
    isLast: false,
  },
  contests:  genContests(10),
  // [
  //   {
  //     id: 1,
  //     title: "丽泽上林入门组训练赛day21",
  //     link: "/contest/1",
  //     startTime: Date.now(),
  //     duration: 1919810,
  //     participants: 114,
  //     registered: false,

  //     totalLikes: 407, // likes - dislikes
  //     like: true,
  //     dislike: false,
  //   },
  // ],
};
</script>

<template>
  <PageContainer>
    <SectionContainer title="比赛列表">
      <div class="p-2">
        <table class="w-full text-sm">
          <thead>
            <TableHeaderRow>
              <th class="text-brand pb-2 w-12">ID</th>
              <th class="text-brand pb-2 text-left">比赛</th>
              <th class="text-brand pb-2 text-center">开始时间/时长</th>
              <th class="text-brand pb-2">报名人数</th>
              <th class="text-brand pb-2">评价</th>
            </TableHeaderRow>
          </thead>
          <tbody>
            <TableRow v-for="p in data.contests" :key="p.id">
              <td class="text-center py-2">{{ p.id }}</td>
              <td class="py-2">
                <TextLink :to="p.link">{{ p.title }}</TextLink>
              </td>
              <td class="py-2 text-center flex flex-col whitespace-nowrap">
                <DateTime :time="p.startTime" />
                <Elapse :elapse="p.duration" />
              </td>
              <td class="py-2 text-center">
                {{ p.participants }}
              </td>
              <td class="py-2 text-left">
                <button>好评</button>
                <button class="mx-1">差评</button>
                {{ p.totalLikes }}
              </td>
            </TableRow>
          </tbody>
        </table>
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
