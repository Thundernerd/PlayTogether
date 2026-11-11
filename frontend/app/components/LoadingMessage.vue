<template>
  <p class="loading-message">
    <LoadingSpinner size="md" />
    <slot />
  </p>
</template>

<style scoped>
.loading-message {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--muted);
}
</style>
