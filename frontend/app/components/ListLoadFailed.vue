<script setup lang="ts">
defineProps<{
  retrying?: boolean
}>()

defineEmits<{
  retry: []
}>()
</script>

<template>
  <div class="list-load-failed">
    <p class="error-message"><slot /></p>
    <button class="btn" type="button" :disabled="retrying" @click="$emit('retry')">
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
