<script setup lang="ts">
import type { Group } from '~/types/api'

defineProps<{
  group: Group
  isMember: boolean
  actionLoading?: boolean
  basePath: string
  wishlistCount?: number
  libraryCount?: number
  sharedCount?: number
}>()

const emit = defineEmits<{
  copyInvite: []
  deleteGroup: []
}>()
</script>

<template>
  <header class="group-header card">
    <div class="header-main">
      <h1>{{ group.name }}</h1>
      <p class="group-invite">
        Invite code <strong>{{ group.invite_code }}</strong>
        <button class="btn linkish" type="button" @click="emit('copyInvite')">Copy link</button>
      </p>
      <GroupNav
        v-if="isMember"
        :base-path="basePath"
        :wishlist-count="wishlistCount"
        :library-count="libraryCount"
        :shared-count="sharedCount"
      />
    </div>
    <button
      v-if="isMember"
      class="btn btn-danger"
      type="button"
      :disabled="actionLoading"
      @click="emit('deleteGroup')"
    >
      Delete group
    </button>
  </header>
</template>

<style scoped>
.group-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 1rem;
}

.header-main {
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
  min-width: 0;
}

.group-header h1 {
  margin: 0;
}

.group-invite {
  margin: 0;
  color: var(--muted);
  font-size: 0.9375rem;
}

.linkish {
  margin-left: 0.75rem;
  font-size: 0.8125rem;
  padding: 0.25rem 0.5rem;
}

.btn-danger {
  border-color: rgba(248, 113, 113, 0.5);
  color: #fca5a5;
  white-space: nowrap;
}

.btn-danger:hover:not(:disabled) {
  border-color: #f87171;
  background: rgba(248, 113, 113, 0.1);
}
</style>
