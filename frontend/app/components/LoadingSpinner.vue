<script setup lang="ts">
withDefaults(
  defineProps<{
    size?: 'sm' | 'md'
  }>(),
  { size: 'sm' },
)
</script>

<template>
  <span
    class="spinner"
    :class="size"
    role="status"
    aria-label="Loading"
  />
</template>

<style scoped>
.spinner {
  display: inline-block;
  border-radius: 50%;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  animation: spin 0.7s linear infinite;
  flex-shrink: 0;
}

.spinner.sm {
  width: 0.75rem;
  height: 0.75rem;
}

.spinner.md {
  width: 1rem;
  height: 1rem;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
