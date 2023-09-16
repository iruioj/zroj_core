<script setup lang="ts">
const data = {
  id: 1,
  title: "丽泽上林入门组训练赛day21",
};

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
      <div class="grow text-2xl text-brand">
        #{{ data.id }} {{ data.title }}
      </div>
    </div>
    <div class="my-4">
      <UBtn @click="onBind">Bind Workding Directory</UBtn>
    </div>
  </PageContainer>
</template>
