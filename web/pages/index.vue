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
            <NuxtLink :to="an.link" class="hover:text-brand grow">{{
              an.title
            }}</NuxtLink>
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
  </div>
</template>
