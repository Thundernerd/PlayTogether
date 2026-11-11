<script setup lang="ts">
import type { WishlistItemView } from '~/types/api'
import { matchesPlayTogetherFilter, type PlayTogetherFilter } from '~/utils/playModes'

type WishlistFilter = 'all' | 'selected' | 'unselected'
type ReleaseFilter = 'all' | 'released' | 'unreleased' | 'unknown'
type WishlistSort = 'name_asc' | 'name_desc' | 'date_desc' | 'date_asc'

const group = useGroup()
const { myWishlist, syncComplete, wishlistsLoading, actionLoading, toggleSelection } = group
const { startPolling, stopPolling } = useSyncStatus(group.onSyncUpdate)
const { open: openGameDetail } = useGameDetail()

const wishlistSearch = ref('')
const wishlistFilter = ref<WishlistFilter>('all')
const releaseFilter = ref<ReleaseFilter>('all')
const playTogetherFilter = ref<PlayTogetherFilter>('all')
const wishlistSort = ref<WishlistSort>('date_desc')

const filteredWishlistItems = computed(() => {
  const items = myWishlist.value?.items ?? []
  const query = wishlistSearch.value.trim().toLowerCase()

  const filtered = items.filter((item) => {
    if (wishlistFilter.value === 'selected' && !item.wants_to_play) return false
    if (wishlistFilter.value === 'unselected' && item.wants_to_play) return false

    const isReleased = item.game?.is_released
    if (releaseFilter.value === 'released' && isReleased !== true) return false
    if (releaseFilter.value === 'unreleased' && isReleased !== false) return false
    if (releaseFilter.value === 'unknown' && isReleased != null) return false

    if (!matchesPlayTogetherFilter(item.game?.play_modes, playTogetherFilter.value)) {
      return false
    }

    if (!query) return true

    const name = item.game?.name?.toLowerCase() ?? ''
    const description = item.game?.short_description?.toLowerCase() ?? ''
    const tags = item.game?.tags?.join(' ').toLowerCase() ?? ''
    return name.includes(query) || description.includes(query) || tags.includes(query)
  })

  return [...filtered].sort(compareWishlistItems)
})

function compareWishlistItems(a: WishlistItemView, b: WishlistItemView) {
  switch (wishlistSort.value) {
    case 'name_asc':
      return (a.game?.name ?? '').localeCompare(b.game?.name ?? '')
    case 'name_desc':
      return (b.game?.name ?? '').localeCompare(a.game?.name ?? '')
    case 'date_asc':
      return a.date_added - b.date_added
    case 'date_desc':
    default:
      return b.date_added - a.date_added
  }
}

watch(
  () => myWishlist.value?.member.sync_status.status,
  (status) => {
    if (status === 'running') startPolling()
  },
)

onMounted(() => {
  if (myWishlist.value?.member.sync_status.status === 'running') {
    startPolling()
  }
})

onUnmounted(stopPolling)
</script>

<template>
  <div class="list-page">
    <section class="card">
      <h2 class="section-title">Your Steam wishlist</h2>
      <p class="section-subtitle">
        Mark games you want to play together. Use search and filters to find titles quickly.
      </p>

      <div v-if="myWishlist?.member.sync_status.status === 'running'" class="sync-banner">
        <SyncStatusBadge :status="myWishlist.member.sync_status" />
      </div>

      <template v-if="myWishlist?.items.length">
        <div class="wishlist-toolbar">
          <input
            v-model="wishlistSearch"
            class="input"
            type="search"
            placeholder="Search by name, description, or tag…"
          />
          <select v-model="wishlistFilter" class="input select">
            <option value="all">All games</option>
            <option value="selected">Want to play together</option>
            <option value="unselected">Not selected</option>
          </select>
          <select v-model="releaseFilter" class="input select">
            <option value="all">Any release status</option>
            <option value="released">Released</option>
            <option value="unreleased">Unreleased</option>
            <option value="unknown">Release unknown</option>
          </select>
          <select v-model="playTogetherFilter" class="input select">
            <option value="all">Any play mode</option>
            <option value="online_coop">Online co-op</option>
            <option value="multiplayer_coop">Multiplayer / co-op</option>
            <option value="unknown">Play mode unknown</option>
          </select>
          <select v-model="wishlistSort" class="input select">
            <option value="date_desc">Date added (newest)</option>
            <option value="date_asc">Date added (oldest)</option>
            <option value="name_asc">Name (A–Z)</option>
            <option value="name_desc">Name (Z–A)</option>
          </select>
        </div>

        <p class="wishlist-count muted">
          Showing {{ filteredWishlistItems.length }} of {{ myWishlist.items.length }} games
        </p>

        <div v-if="filteredWishlistItems.length" class="grid-cards">
          <GameCard
            v-for="item in filteredWishlistItems"
            :key="item.appid"
            :appid="item.appid"
            :game="item.game"
            :wants-to-play="item.wants_to_play"
            :loading="!syncComplete && !item.game"
            show-checkbox
            :disabled="!syncComplete || actionLoading"
            @toggle="toggleSelection(item.appid, $event)"
            @open="openGameDetail(item.appid)"
          />
        </div>
        <p v-else class="muted">No games match your search or filters.</p>
      </template>

      <LoadingMessage v-else-if="wishlistsLoading">Loading your wishlist…</LoadingMessage>
      <p v-else-if="syncComplete" class="muted">Your wishlist is empty.</p>
      <p v-else class="muted">Sync your Steam wishlist from the overview page first.</p>
    </section>

    <ScrollToTop />
    <GameDetailModal />
  </div>
</template>

<style scoped>
.list-page {
  padding-bottom: 5rem;
}

.sync-banner {
  margin: 0.75rem 0;
}

.wishlist-toolbar {
  display: grid;
  grid-template-columns: 1fr repeat(4, auto);
  gap: 0.5rem;
  margin-top: 1rem;
}

.wishlist-count {
  margin: 0.5rem 0 1rem;
  font-size: 0.8125rem;
}

.select {
  min-width: 11rem;
}

.muted {
  color: var(--muted);
}

@media (max-width: 720px) {
  .wishlist-toolbar {
    grid-template-columns: 1fr;
  }

  .select {
    min-width: 0;
  }
}
</style>
