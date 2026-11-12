<script setup lang="ts">
// A list that never loaded: stands in for its items. Try again stays enabled while a request is
// in flight, for the reason given in ListRefreshFailed.
defineEmits<{
  retry: []
}>()
</script>

<template>
  <div class="list-load-failed">
    <p class="error-message"><slot /></p>
    <button class="btn" type="button" @click="$emit('retry')">
      Try again
    </button>
  </div>
</template>

<style scoped>
.list-load-failed {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.75rem;
  margin-top: 1rem;
}
</style>
