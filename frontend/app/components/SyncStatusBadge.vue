<script setup lang="ts">
import type { SyncStatus } from '~/types/api'

const props = defineProps<{
  status: SyncStatus
}>()

const label = computed(() => {
  const s = props.status
  if (s.status === 'running') {
    if (s.phase === 'fetching_owned') return 'Fetching library…'
    if (s.phase === 'fetching_wishlist') return 'Fetching wishlist…'
    if (s.phase === 'fetching_games') {
      if (s.progress_total > 0) {
        return `Fetching game data ${s.progress_done}/${s.progress_total}`
      }
      return 'Fetching game data…'
    }
    return 'Syncing…'
  }
  if (s.status === 'complete') return 'Ready'
  if (s.status === 'error') return s.error_message || 'Sync failed'
  return 'Idle'
})

const tone = computed(() => {
  switch (props.status.status) {
    case 'running':
      return 'running'
    case 'complete':
      return 'complete'
    case 'error':
      return 'error'
    default:
      return 'idle'
  }
})
</script>

<template>
  <span class="badge" :class="tone">{{ label }}</span>
</template>

<style scoped>
.badge {
  display: inline-flex;
  align-items: center;
  padding: 0.2rem 0.55rem;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 700;
  border: var(--line);
  background: var(--surface);
  color: var(--ink);
  max-width: 100%;
  word-break: break-word;
}

.running {
  background: var(--warning-tint);
  color: var(--warning-text);
}

.complete {
  background: var(--success-tint);
  color: var(--success-text);
}

.error {
  background: var(--danger-tint);
  color: var(--danger-text);
}

.idle {
  color: var(--muted);
}
</style>
