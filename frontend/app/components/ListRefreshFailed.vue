<script setup lang="ts">
// A loaded list's last refresh failed: a non-blocking line above the items the page still
// shows (unlike ListLoadFailed, which stands in for a list that never loaded).
defineProps<{
  retrying?: boolean
}>()

defineEmits<{
  retry: []
}>()
</script>

<template>
  <div class="list-refresh-failed" role="status">
    <p class="refresh-failed-message">Couldn't refresh: showing the last loaded list.</p>
    <button class="btn" type="button" :disabled="retrying" @click="$emit('retry')">
      Try again
    </button>
  </div>
</template>

<style scoped>
.list-refresh-failed {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.75rem;
  margin: 1rem 0 0;
  padding: 0.5rem 0.75rem;
  border-radius: var(--radius-sm);
  background: var(--warning-tint);
}

.refresh-failed-message {
  margin: 0;
  color: var(--warning-text);
  font-weight: 700;
}
</style>
