<script setup lang="ts">
const id = computed(() => parseInt(useRoute().params.id as string));
const { data } = await useAPI().contest.info.get.use({ id: id.value });

async function* allfiles(
  dirHandle: any,
  prefix = "",
): AsyncGenerator<[string, FileSystemFileHandle]> {
  prefix += dirHandle.name + "/";
  for await (const [_, value] of dirHandle.entries()) {
    if (value.kind === "file") {
      yield [prefix + value.name, value];
    } else {
      yield* allfiles(value, prefix);
    }
  }
}

const onBind = async () => {
  if ("showDirectoryPicker" in window) {
    const picker: any = window.showDirectoryPicker;
    const dirHandle: any = await picker();
    // for await (const [key, value] of dirHandle.entries()) {
    //   console.log({ key, value })
    // }
    for await (const [path, file] of allfiles(dirHandle)) {
      console.log(path);
    }
  }
};
</script>

<template>
  <PageContainer>
    <div class="mt-4">
      <div v-if="data" class="grow text-2xl text-brand">
        #{{ data.meta.id }} {{ data.meta.title }}
      </div>
    </div>
    <pre v-if="data">{{ data.problems }}</pre>
    <div class="my-4">
      <UBtn @click="onBind">Bind Workding Directory</UBtn>
    </div>
  </PageContainer>
</template>
