<script setup lang="ts">
const group = useGroup()
const {
  me,
  myWishlist,
  myLibrary,
  sharedGames,
  syncComplete,
  anyMemberSyncing,
  librariesLoading,
  wishlistsLoading,
  sharedLoading,
  librariesError,
  wishlistsError,
  sharedError,
  actionLoading,
  saveSteamId,
  refreshWishlistSync,
  groupDetail,
  groupBasePath,
} = group

const steamIdInput = ref('')

watch(
  () => me.value?.member.steam_id,
  (steamId) => {
    if (steamId) {
      steamIdInput.value = String(steamId)
    }
  },
  { immediate: true },
)
</script>

<template>
  <div class="overview-page">
    <section class="card">
      <h2 class="section-title">Members</h2>
      <MemberList v-if="groupDetail?.members" :members="groupDetail.members" />
    </section>

    <section class="card">
      <h2 class="section-title">Steam setup</h2>
      <p class="section-subtitle">
        Your Steam profile, wishlist, and game details must be public. Enter your Steam ID64 to sync.
      </p>

      <div class="steam-form">
        <input v-model="steamIdInput" class="input" placeholder="76561198000000000" />
        <button
          class="btn btn-primary"
          :disabled="actionLoading"
          @click="saveSteamId(steamIdInput)"
        >
          Save & sync
        </button>
        <button class="btn" :disabled="actionLoading || !me?.member.steam_id" @click="refreshWishlistSync">
          Refresh Steam data
        </button>
      </div>

      <div v-if="myWishlist?.member.sync_status.status === 'running'" class="sync-banner">
        <SyncStatusBadge :status="myWishlist.member.sync_status" />
      </div>
    </section>

    <section class="card quick-links">
      <h2 class="section-title">Explore</h2>
      <!-- A failed list says so instead of "0 games". It is checked before loading so the
           marker holds steady while a sync keeps polling a failing endpoint. -->
      <div class="link-grid">
        <NuxtLink :to="`${groupBasePath}/library`" class="quick-link card">
          <strong>My library</strong>
          <span v-if="librariesError" class="count-failed">Couldn't load</span>
          <span v-else-if="librariesLoading" class="count-loading">
            <LoadingSpinner size="sm" /> Loading…
          </span>
          <span v-else>{{ myLibrary?.items.length ?? 0 }} games</span>
          <span class="hint">Browse owned games and mark titles to play together</span>
        </NuxtLink>
        <NuxtLink :to="`${groupBasePath}/wishlist`" class="quick-link card">
          <strong>My wishlist</strong>
          <span v-if="wishlistsError" class="count-failed">Couldn't load</span>
          <span v-else-if="wishlistsLoading" class="count-loading">
            <LoadingSpinner size="sm" /> Loading…
          </span>
          <span v-else>{{ myWishlist?.items.length ?? 0 }} games</span>
          <span class="hint">Browse, filter, and mark games to play together</span>
        </NuxtLink>
        <NuxtLink :to="`${groupBasePath}/play-together`" class="quick-link card">
          <strong>Play together</strong>
          <span v-if="sharedError" class="count-failed">Couldn't load</span>
          <span v-else-if="sharedLoading" class="count-loading">
            <LoadingSpinner size="sm" /> Loading…
          </span>
          <span v-else>{{ sharedGames.length }} games</span>
          <span class="hint">Vote and rate the group's shared picks</span>
        </NuxtLink>
      </div>
      <p v-if="!me?.member.steam_id" class="muted">Add your Steam ID above to sync your library and wishlist.</p>
      <p v-else-if="!syncComplete && anyMemberSyncing" class="muted">Sync in progress…</p>
    </section>
  </div>
</template>

<style scoped>
.overview-page {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.steam-form {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.steam-form .input {
  flex: 1 1 220px;
}

.sync-banner {
  margin-top: 0.75rem;
}

.link-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1rem;
}

.quick-link {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  text-decoration: none;
  color: inherit;
  transition: background 0.15s;
}

.quick-link:hover {
  background: var(--highlight-tint);
}

.quick-link strong {
  font-size: 1.0625rem;
}

.count-loading {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  color: var(--muted);
}

.count-failed {
  color: var(--danger-text);
  font-weight: 700;
}

.hint {
  color: var(--muted);
  font-size: 0.875rem;
}

.muted {
  color: var(--muted);
  margin: 0.75rem 0 0;
}
</style>
