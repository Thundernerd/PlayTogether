<script setup lang="ts">
import type { CountStatus } from '~/utils/listStatus'

// A list's game count on an overview quick link. A list that never loaded says so instead of
// "0 games"; one whose last refresh failed keeps its last count with a "couldn't refresh" hint.
defineProps<{
  status: CountStatus
  count: number
}>()
</script>

<template>
  <span v-if="status === 'failed'" class="count-failed">Couldn't load</span>
  <span v-else-if="status === 'loading'" class="count-loading">
    <LoadingSpinner size="sm" /> Loading…
  </span>
  <span v-else>
    {{ count }} games
    <span v-if="status === 'stale'" class="count-stale">· Couldn't refresh</span>
  </span>
</template>

<style scoped>
.count-loading {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  color: var(--muted);
}

.count-failed {
  color: var(--danger-text);
  font-weight: 700;
}

.count-stale {
  color: var(--warning-text);
  font-size: 0.875rem;
  font-weight: 700;
}
</style>
