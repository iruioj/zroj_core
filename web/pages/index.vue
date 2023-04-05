<script setup lang="ts">
import SectionContainer from "~/components/SectionContainer.vue";

const announcements = [
  {
    id: 1,
    title: "正睿OJ 训练使用说明（2022）",
    link: "http://www.zhengruioi.com/blog/2",
    author: "zhengruioi",
    author_link: "http://www.zhengruioi.com/user/profile/zhengruioi",
    date: "2022-03-04 21:46:54",
  },
  {
    id: 2,
    title: "ZROJ 更新日志（出题人最好时常看一看）",
    link: "http://www.zhengruioi.com/blog/2",
    author: "zhengruioi",
    author_link: "http://www.zhengruioi.com/user/profile/zhengruioi",
    date: "2022-03-04 21:46:54",
  },
  {
    id: 3,
    title: "正睿报名、合作、投题联系方式",
    link: "http://www.zhengruioi.com/blog/2",
    author: "zhengruioi",
    author_link: "http://www.zhengruioi.com/user/profile/zhengruioi",
    date: "2022-03-04 21:46:54",
  },
];

const topUsers = [
  {
    rank: 1,
    username: "larryzhong",
    rating: 2807,
    aleph_count: 2,
    motto: "",
    link: "http://www.zhengruioi.com/user/profile/larryzhong",
  },
  {
    rank: 2,
    username: "byqx",
    rating: 2774,
    aleph_count: 2,
    motto: "活不下去了。",
    link: "http://www.zhengruioi.com/user/profile/larryzhong",
  },
  {
    rank: 3,
    username: "huzhaoyang",
    rating: 2686,
    aleph_count: 1,
    motto: "",
    link: "http://www.zhengruioi.com/user/profile/larryzhong",
  },
  {
    rank: 4,
    username: "he_____he",
    rating: 2674,
    aleph_count: 1,
    motto: "",
    link: "http://www.zhengruioi.com/user/profile/larryzhong",
  },
];
</script>

<template>
  <div class="w-[700px] m-auto">
    <SectionContainer title="公告">
      <ul class="p-2">
        <li
          v-for="an in announcements"
          :key="an.id"
          class="py-2 border-b border-dashed border-b-black/20 last:border-b-0"
        >
          <div class="inline-block flex">
            <NuxtLink
              :to="an.link"
              class="text-brand-dark grow underline decoration-brand/0 hover:decoration-brand underline-offset-4 transition ease-in-out duration-200"
            >
              {{ an.title }}</NuxtLink
            >
            <div>
              by
              <NuxtLink
                :to="an.author_link"
                class="font-bold text-black hover:text-brand"
                >{{ an.author }}
              </NuxtLink>
            </div>
            <div class="pl-2">更新于 {{ an.date }}</div>
          </div>
        </li>
      </ul>
    </SectionContainer>

    <SectionContainer title="排行榜（活跃用户）">
      <div class="p-2">
        <table class="w-full">
          <thead class="text-brand-dark">
            <th>#</th>
            <th>用户</th>
            <th>格言</th>
            <th>Rating</th>
          </thead>
          <tr
            v-for="u in topUsers"
            :key="u.username"
            class="border-b border-dashed border-b-black/20 last:border-b-0"
          >
            <td class="py-2 text-center w-8">{{ u.rank }}</td>
            <td class="p-2 text-center w-4">
              <NuxtLink :to="u.link" class="font-bold hover:text-brand"
                >{{ u.username
                }}<sup v-if="u.aleph_count"
                  >ℵ<sup v-if="u.aleph_count > 1">{{ u.aleph_count }}</sup></sup
                ></NuxtLink
              >
            </td>
            <td class="py-2 text-center">{{ u.motto }}</td>
            <td class="py-2 text-center w-12">{{ u.rating }}</td>
          </tr>
        </table>
      </div>
    </SectionContainer>
  </div>
</template>
