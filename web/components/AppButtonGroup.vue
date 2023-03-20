<template>
  <div class="btn-group">
    <slot></slot>
  </div>
</template>

<style>
.btn-group > .btn {
  @apply border-0 border-y border-l last:border-r
    rounded-none first:rounded-l last:rounded-r;
}
</style>
