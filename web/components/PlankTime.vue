<script lang="ts" setup>
const props = defineProps<{
  seconds: number;
  exact?: boolean;
}>();

// https://physics.nist.gov/cgi-bin/cuu/Value?plkt
const tp = 5.3912476e-44;

const num = computed(() => {
  const [mantissa, exponent] = (props.seconds / tp).toExponential(3).split("+");
  return [parseFloat(mantissa), parseInt(exponent)];
});
</script>
<template>
  <div>
    <span v-if="!props.exact">≈</span>{{ num[0] }}×10<sup>{{ num[1] }}</sup>
    <span title="普朗克时间">PTU</span>
  </div>
</template>
