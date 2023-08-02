<script lang="ts" setup>
import Code from "./Code.vue";

import Heading from "./Heading.vue";
import InlineMath from "./InlineMath.vue";
import Paragraph from "./Paragraph.vue";
import Root from "./Root.vue";
import Text from "./Text.vue";
import TwoColumns from "./TwoColumns.vue";
import { Node } from "composables/api";

defineProps<{
  data: Node;
}>();
</script>

<template>
  <Root v-if="data.type == 'root'" :data="data" />
  <Heading v-else-if="data.type == 'heading'" :data="data" />
  <Text v-else-if="data.type == 'text'" :data="data" />
  <Paragraph v-else-if="data.type == 'paragraph'" :data="data" />
  <Code v-else-if="data.type == 'code'" :data="data" />
  <InlineMath v-else-if="data.type == 'inlineMath'" :data="data" />
  <TwoColumns v-else-if="data.type == 'twoColumns'" :data="data" />
  <pre v-else>{{ data }}</pre>
</template>
