<script setup lang="ts">
import type { SyncStatus } from '~/types/api'

const props = defineProps<{
  status: SyncStatus
}>()

const label = computed(() => {
  const s = props.status
  if (s.status === 'running') {
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
  border-radius: 999px;
  font-size: 0.75rem;
  border: 1px solid var(--border);
  background: var(--surface-2);
  max-width: 100%;
  word-break: break-word;
}

.running {
  border-color: rgba(56, 189, 248, 0.5);
  color: #7dd3fc;
}

.complete {
  border-color: rgba(34, 197, 94, 0.5);
  color: #86efac;
}

.error {
  border-color: rgba(248, 113, 113, 0.5);
  color: #fca5a5;
}

.idle {
  color: var(--muted);
}
</style>
