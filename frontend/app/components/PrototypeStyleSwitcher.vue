<script setup lang="ts">
// PROTOTYPE (issue #1): floating style switcher. Only mounted when import.meta.dev.
import { PROTOTYPE_STYLES, resolvePrototypeStyle } from '~/utils/prototypeStyles'

const route = useRoute()
const router = useRouter()

const index = computed(() => {
  const key = resolvePrototypeStyle(route.query.style)
  return PROTOTYPE_STYLES.findIndex((s) => s.key === key)
})
const current = computed(() => PROTOTYPE_STYLES[index.value]!)

function cycle(delta: number) {
  const n = PROTOTYPE_STYLES.length
  const next = PROTOTYPE_STYLES[(index.value + delta + n) % n]!
  router.replace({ query: { ...route.query, style: next.key } })
}

function isTyping(target: EventTarget | null) {
  if (!(target instanceof HTMLElement)) return false
  return (
    target.closest('input, textarea, select, [contenteditable]:not([contenteditable="false"])') != null
  )
}

function onKeydown(event: KeyboardEvent) {
  if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return
  if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return
  if (isTyping(event.target)) return
  // The game detail modal uses ←/→ for its media carousel; leave them to it.
  if (document.querySelector('[aria-modal="true"]')) return
  cycle(event.key === 'ArrowLeft' ? -1 : 1)
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="proto-switcher" role="toolbar" aria-label="Prototype style switcher">
    <button type="button" class="proto-arrow" aria-label="Previous style" @click="cycle(-1)">←</button>
    <div class="proto-label">
      <span class="proto-title">
        <span class="proto-tag">PROTOTYPE</span>
        {{ current.key === 'current' ? 'current' : current.key.toUpperCase() }} · {{ current.name }}
      </span>
      <span class="proto-rationale">{{ current.rationale }}</span>
    </div>
    <button type="button" class="proto-arrow" aria-label="Next style" @click="cycle(1)">→</button>
  </div>
</template>

<style scoped>
/* Deliberately outside every style under review: system font, fixed colours,
   caution-tape frame. No design tokens are used here. */
.proto-switcher {
  position: fixed;
  left: 50%;
  /* Sits above the Nuxt DevTools pill, which is also bottom-centre in dev. */
  bottom: 3rem;
  transform: translateX(-50%);
  z-index: 2000;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: max-content;
  max-width: min(640px, calc(100vw - 7rem));
  padding: 0.4rem;
  border-radius: 14px;
  background:
    linear-gradient(#111, #111) padding-box,
    repeating-linear-gradient(45deg, #ffd400 0 8px, #111 8px 16px) border-box;
  border: 3px solid transparent;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.45);
  color: #fff;
  font: 13px/1.35 ui-sans-serif, -apple-system, 'Segoe UI', Roboto, sans-serif;
  letter-spacing: normal;
  text-transform: none;
}

.proto-arrow {
  flex: 0 0 auto;
  width: 2.25rem;
  height: 2.25rem;
  border: none;
  border-radius: 10px;
  background: #ffd400;
  color: #111;
  font: 700 16px/1 ui-sans-serif, -apple-system, sans-serif;
  cursor: pointer;
}

.proto-arrow:hover {
  background: #fff;
}

.proto-arrow:focus-visible {
  outline: 2px solid #fff;
  outline-offset: 2px;
}

.proto-label {
  display: flex;
  flex-direction: column;
  min-width: 0;
  padding: 0 0.25rem;
}

.proto-title {
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.proto-tag {
  display: inline-block;
  margin-right: 0.35rem;
  padding: 0 0.3rem;
  border-radius: 4px;
  background: #ffd400;
  color: #111;
  font-size: 10px;
  letter-spacing: 0.06em;
  vertical-align: 1px;
}

.proto-rationale {
  color: #d4d4d4;
  font-size: 12px;
}
</style>
