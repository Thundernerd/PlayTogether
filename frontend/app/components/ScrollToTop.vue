<script setup lang="ts">
const visible = ref(false)

function onScroll() {
  visible.value = window.scrollY > 400
}

function scrollToTop() {
  window.scrollTo({ top: 0, behavior: 'smooth' })
}

onMounted(() => {
  onScroll()
  window.addEventListener('scroll', onScroll, { passive: true })
})

onUnmounted(() => {
  window.removeEventListener('scroll', onScroll)
})
</script>

<template>
  <button
    v-show="visible"
    type="button"
    class="scroll-top btn"
    aria-label="Scroll to top"
    @click="scrollToTop"
  >
    ↑ Top
  </button>
</template>

<style scoped>
.scroll-top {
  position: fixed;
  right: 1.25rem;
  bottom: 1.25rem;
  z-index: 50;
  background: var(--floating-bg);
}
</style>
