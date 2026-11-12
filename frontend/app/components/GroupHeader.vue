<script setup lang="ts">
import type { Group } from '~/types/api'
import type { CountStatus } from '~/utils/listStatus'

defineProps<{
  group: Group
  isMember: boolean
  actionLoading?: boolean
  basePath: string
  wishlistCount?: number
  libraryCount?: number
  sharedCount?: number
  libraryStatus?: CountStatus
  wishlistStatus?: CountStatus
  sharedStatus?: CountStatus
}>()

const emit = defineEmits<{
  copyInvite: []
  leaveGroup: []
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
        :library-status="libraryStatus"
        :wishlist-status="wishlistStatus"
        :shared-status="sharedStatus"
      />
    </div>
    <div v-if="isMember" class="header-actions">
      <button
        class="btn"
        type="button"
        :disabled="actionLoading"
        @click="emit('leaveGroup')"
      >
        Leave group
      </button>
      <button
        class="btn btn-danger"
        type="button"
        :disabled="actionLoading"
        @click="emit('deleteGroup')"
      >
        Delete group
      </button>
    </div>
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
  box-shadow: var(--shadow-sm);
}

.btn-danger {
  color: var(--danger-text);
  white-space: nowrap;
}

.header-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  justify-content: flex-end;
}

.btn-danger:hover:not(:disabled) {
  background: var(--danger-tint);
}
</style>
