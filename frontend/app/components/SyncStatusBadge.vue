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
  border-radius: var(--radius-pill);
  font-size: 0.75rem;
  border: 1px solid var(--border);
  background: var(--surface-2);
  max-width: 100%;
  word-break: break-word;
}

.running {
  border-color: var(--accent-border);
  color: var(--accent-text);
}

.complete {
  border-color: var(--success-border);
  color: var(--success-text);
}

.error {
  border-color: var(--danger-border);
  color: var(--danger-text);
}

.idle {
  color: var(--muted);
}
</style>
