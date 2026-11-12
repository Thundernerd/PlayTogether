<script setup lang="ts">
import type { ListStatus } from '~/utils/listStatus'

// What a list page shows in place of its items, for every status but 'items' (which renders
// nothing). Lists without a sync never reach 'syncing' or 'not-synced', so those texts are
// optional; the empty text can be replaced with the `empty` slot when it needs markup.
defineProps<{
  status: ListStatus
  loadingText: string
  failedText: string
  syncingText?: string
  emptyText?: string
  notSyncedText?: string
}>()

defineEmits<{
  retry: []
}>()
</script>

<template>
  <LoadingMessage v-if="status === 'loading'">{{ loadingText }}</LoadingMessage>
  <ListLoadFailed v-else-if="status === 'failed'" @retry="$emit('retry')">
    {{ failedText }}
  </ListLoadFailed>
  <p v-else-if="status === 'syncing'" class="muted">{{ syncingText }}</p>
  <p v-else-if="status === 'empty'" class="muted">
    <slot name="empty">{{ emptyText }}</slot>
  </p>
  <p v-else-if="status === 'not-synced'" class="muted">{{ notSyncedText }}</p>
</template>

<style scoped>
.muted {
  color: var(--muted);
}
</style>
