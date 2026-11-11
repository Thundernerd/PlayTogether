<script setup lang="ts">
import type { MemberPublic } from '~/types/api'

defineProps<{
  members: MemberPublic[]
}>()
</script>

<template>
  <div class="member-list">
    <div v-for="member in members" :key="member.id" class="member-row">
      <div>
        <strong>{{ member.display_name }}</strong>
        <div class="meta">
          Steam ID:
          <span v-if="member.steam_id">{{ member.steam_id }}</span>
          <span v-else class="muted">not set</span>
        </div>
      </div>
      <SyncStatusBadge :status="member.sync_status" />
    </div>
  </div>
</template>

<style scoped>
.member-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.member-row {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  align-items: flex-start;
  padding: 0.75rem;
  border: var(--line-dashed);
  border-radius: var(--radius-sm);
  background: var(--surface-bright);
}

.meta {
  font-size: 0.8125rem;
  color: var(--muted);
  margin-top: 0.25rem;
}

.muted {
  font-style: italic;
}
</style>
