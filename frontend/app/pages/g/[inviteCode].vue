<script setup lang="ts">
const route = useRoute()
const group = useGroup()
const {
  groupDetail,
  me,
  myWishlist,
  myLibrary,
  isMember,
  anyMemberSyncing,
  loading,
  actionLoading,
  error,
  listError,
  groupBasePath,
  ensureLoaded,
  joinGroup,
  leaveGroup,
  deleteGroup,
  copyInvite,
  onSyncUpdate,
} = group
const { countStatus: librariesCountStatus } = useMemberList('libraries')
const { countStatus: wishlistsCountStatus } = useMemberList('wishlists')
const { data: sharedGames, countStatus: sharedCountStatus } = useMemberList('shared')

const joinDisplayName = ref('')

// List failures show here only on pages without their own failed state (the overview).
const layoutError = computed(
  () => error.value ?? (route.meta.showsListErrors ? null : listError.value),
)
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
      :library-status="librariesCountStatus"
      :wishlist-status="wishlistsCountStatus"
      :shared-status="sharedCountStatus"
      @copy-invite="copyInvite"
      @leave-group="leaveGroup"
      @delete-group="deleteGroup"
    />

    <p v-if="layoutError" class="error-message">{{ layoutError }}</p>

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

.join-panel {
  max-width: 420px;
}
</style>
