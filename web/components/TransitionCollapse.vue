<!-- 主体不要随便加 Y 轴的 padding，可能会影响到动画。必要时考虑 box-border -->
<script lang="ts">
export default {
  props: {
    duration: {
      type: Number,
      default() {
        return 300;
      },
    },
  },
  computed: {
    transStyle() {
      return `all ${this.duration}ms ease`;
    },
  },
  methods: {
    // called before the element is inserted into the DOM.
    // use this to set the "enter-from" state of the element
    onBeforeEnter(el: Element) {
      (el as HTMLElement).style.maxHeight = "0px";
      // console.log("before enter", el);
    },

    // called one frame after the element is inserted.
    // use this to start the animation.
    onEnter(el: Element, done: any) {
      // console.log("enter begin");
      (el as HTMLElement).style.maxHeight = el.scrollHeight + 20 + "px";
      // call the done callback to indicate transition end
      // optional if used in combination with CSS
      // console.log("enter done");
      setTimeout(done, this.duration);
    },

    // called when the enter transition has finished.
    onAfterEnter(el: Element) {
      // console.log("after enter");
      (el as HTMLElement).style.removeProperty("max-height");
    },
    onEnterCancelled(_el: Element) {
      // console.log("cancel enter");
    },

    // called before the leave hook.
    // Most of the time, you should just use the leave hook.
    onBeforeLeave(el: Element) {
      // console.log("before leave");
      (el as HTMLElement).style.maxHeight = el.scrollHeight + 20 + "px";
    },

    // called when the leave transition starts.
    // use this to start the leaving animation.
    onLeave(el: Element, done: any) {
      // console.log("leave begin");
      (el as HTMLElement).style.maxHeight = "0px";
      // call the done callback to indicate transition end
      // optional if used in combination with CSS
      // console.log("leave done");
      setTimeout(done, this.duration);
    },

    // called when the leave transition has finished and the
    // element has been removed from the DOM.
    onAfterLeave(el: Element) {
      // console.log("after leave");
      (el as HTMLElement).style.removeProperty("max-height");
    },

    // only available with v-show transitions
    onLeaveCancelled(_el: Element) {
      // console.log("cancel leave");
    },
  },
};
</script>

<template>
  <Transition
    name="collapse"
    appear
    @before-enter="onBeforeEnter"
    @enter="onEnter"
    @after-enter="onAfterEnter"
    @enter-cancelled="onEnterCancelled"
    @before-leave="onBeforeLeave"
    @leave="onLeave"
    @after-leave="onAfterLeave"
    @leave-cancelled="onLeaveCancelled"
  >
    <slot></slot>
  </Transition>
</template>

<style>
.collapse-enter-active,
.collapse-leave-active {
  transition: v-bind(transStyle);
  overflow-y: hidden;
}
</style>
