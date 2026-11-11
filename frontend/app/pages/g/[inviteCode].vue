<script setup lang="ts">
const group = useGroup()
const {
  groupDetail,
  me,
  myWishlist,
  myLibrary,
  sharedGames,
  isMember,
  anyMemberSyncing,
  loading,
  librariesLoading,
  wishlistsLoading,
  sharedLoading,
  actionLoading,
  error,
  groupBasePath,
  ensureLoaded,
  joinGroup,
  leaveGroup,
  deleteGroup,
  copyInvite,
  onSyncUpdate,
} = group

const joinDisplayName = ref('')
const { startPolling, stopPolling } = useSyncStatus(onSyncUpdate)

const wishlistCount = computed(() => myWishlist.value?.items.length ?? 0)
const libraryCount = computed(() => myLibrary.value?.items.length ?? 0)
const sharedCount = computed(() => sharedGames.value.length)

let groupPollTimer: ReturnType<typeof setInterval> | null = null

function startGroupPolling() {
  if (groupPollTimer) return
  groupPollTimer = setInterval(async () => {
    if (!anyMemberSyncing.value) {
      if (groupPollTimer) clearInterval(groupPollTimer)
      groupPollTimer = null
      return
    }
    await group.refreshMemberData()
  }, 2000)
}

watch(anyMemberSyncing, (syncing) => {
  if (syncing) startGroupPolling()
})

watch(
  () => myWishlist.value?.member.sync_status.status,
  (status) => {
    if (status === 'running') {
      startPolling()
    }
  },
)

onMounted(() => ensureLoaded())
onUnmounted(() => {
  stopPolling()
  if (groupPollTimer) clearInterval(groupPollTimer)
})
</script>

<template>
  <div v-if="loading && !groupDetail" class="loading">Loading group…</div>

  <div v-else-if="!groupDetail" class="error-block">Group not found.</div>

  <div v-else class="group-layout">
    <GroupHeader
      :group="groupDetail.group"
      :is-member="isMember"
      :action-loading="actionLoading"
      :base-path="groupBasePath"
      :wishlist-count="wishlistCount"
      :library-count="libraryCount"
      :shared-count="sharedCount"
      :library-loading="librariesLoading"
      :wishlist-loading="wishlistsLoading"
      :shared-loading="sharedLoading"
      @copy-invite="copyInvite"
      @leave-group="leaveGroup"
      @delete-group="deleteGroup"
    />

    <p v-if="error" class="error">{{ error }}</p>

    <section v-if="!isMember" class="card join-panel">
      <h2 class="section-title">Join this group</h2>
      <label class="label">Your display name</label>
      <input v-model="joinDisplayName" class="input" placeholder="Your name" />
      <button
        class="btn btn-primary"
        style="margin-top: 0.75rem"
        :disabled="actionLoading || !joinDisplayName"
        @click="joinGroup(joinDisplayName)"
      >
        Join group
      </button>
    </section>

    <NuxtPage v-else />
  </div>
</template>

<style scoped>
.group-layout {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.loading,
.error-block {
  color: var(--muted);
}

.error {
  color: var(--danger-text);
  margin: 0;
}

.join-panel {
  max-width: 420px;
}
</style>
