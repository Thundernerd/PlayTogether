<script setup lang="ts">
import type { OwnedGameItemView } from '~/types/api'
import { matchesPlayTogetherFilter, type PlayTogetherFilter } from '~/utils/playModes'

type LibraryFilter = 'all' | 'selected' | 'unselected'
type ReleaseFilter = 'all' | 'released' | 'unreleased' | 'unknown'
type LibrarySort =
  | 'playtime_desc'
  | 'playtime_asc'
  | 'last_played_desc'
  | 'name_asc'
  | 'name_desc'

// This page shows its own failed state, so the layout skips the raw list error line.
definePageMeta({ showsListErrors: true })

const group = useGroup()
const {
  myLibrary,
  librarySyncComplete,
  actionLoading,
  toggleSelection,
  retryListLoads,
} = group
const {
  loaded: librariesLoaded,
  loading: librariesLoading,
  error: librariesError,
  refreshFailed: librariesRefreshFailed,
} = useMemberList('libraries')
const { startPolling, stopPolling } = useSyncStatus(group.onSyncUpdate)
const { open: openGameDetail } = useGameDetail()

const librarySearch = ref('')
const libraryFilter = ref<LibraryFilter>('all')
const releaseFilter = ref<ReleaseFilter>('all')
const playTogetherFilter = ref<PlayTogetherFilter>('all')
const librarySort = ref<LibrarySort>('playtime_desc')

const status = computed(() =>
  listStatus({
    itemCount: myLibrary.value?.items.length ?? 0,
    loaded: librariesLoaded.value,
    loading: librariesLoading.value,
    error: librariesError.value,
    sync: {
      status: myLibrary.value?.member.sync_status.status,
      complete: librarySyncComplete.value,
    },
  }),
)

function formatPlaytime(minutes: number) {
  if (minutes <= 0) return 'Never played'
  const hours = Math.floor(minutes / 60)
  if (hours < 1) return `${minutes} min played`
  return `${hours.toLocaleString()} hrs played`
}

function playtimeSubtitle(item: OwnedGameItemView) {
  return formatPlaytime(item.playtime_forever)
}

const filteredLibraryItems = computed(() => {
  const items = myLibrary.value?.items ?? []
  const query = librarySearch.value.trim().toLowerCase()

  const filtered = items.filter((item) => {
    if (libraryFilter.value === 'selected' && !item.wants_to_play) return false
    if (libraryFilter.value === 'unselected' && item.wants_to_play) return false

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

  return [...filtered].sort(compareLibraryItems)
})

function compareLibraryItems(a: OwnedGameItemView, b: OwnedGameItemView) {
  switch (librarySort.value) {
    case 'name_asc':
      return (a.game?.name ?? '').localeCompare(b.game?.name ?? '')
    case 'name_desc':
      return (b.game?.name ?? '').localeCompare(a.game?.name ?? '')
    case 'playtime_asc':
      return a.playtime_forever - b.playtime_forever
    case 'last_played_desc':
      return (b.rtime_last_played ?? 0) - (a.rtime_last_played ?? 0)
    case 'playtime_desc':
    default:
      return b.playtime_forever - a.playtime_forever
  }
}

watch(
  () => myLibrary.value?.member.sync_status.status,
  (status) => {
    if (status === 'running') startPolling()
  },
)

onMounted(() => {
  if (myLibrary.value?.member.sync_status.status === 'running') {
    startPolling()
  }
})

onUnmounted(stopPolling)
</script>

<template>
  <div class="list-page">
    <section class="card">
      <h2 class="section-title">Your Steam library</h2>
      <p class="section-subtitle">
        Browse games you own and mark titles you want to play together.
      </p>

      <div v-if="myLibrary?.member.sync_status.status === 'running'" class="sync-banner">
        <SyncStatusBadge :status="myLibrary.member.sync_status" />
      </div>

      <ListRefreshFailed v-if="librariesRefreshFailed" @retry="retryListLoads" />

      <template v-if="status === 'items' && myLibrary">
        <div class="library-toolbar">
          <input
            v-model="librarySearch"
            class="input"
            type="search"
            placeholder="Search by name, description, or tag…"
          />
          <select v-model="libraryFilter" class="input select">
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
          <select v-model="librarySort" class="input select">
            <option value="playtime_desc">Playtime (most)</option>
            <option value="playtime_asc">Playtime (least)</option>
            <option value="last_played_desc">Last played (newest)</option>
            <option value="name_asc">Name (A–Z)</option>
            <option value="name_desc">Name (Z–A)</option>
          </select>
        </div>

        <p class="library-count muted">
          Showing {{ filteredLibraryItems.length }} of {{ myLibrary.items.length }} games
        </p>

        <div v-if="filteredLibraryItems.length" class="grid-cards">
          <GameCard
            v-for="item in filteredLibraryItems"
            :key="item.appid"
            :appid="item.appid"
            :game="item.game"
            :subtitle="playtimeSubtitle(item)"
            :wants-to-play="item.wants_to_play"
            :loading="!librarySyncComplete && !item.game"
            show-checkbox
            :disabled="!librarySyncComplete || actionLoading"
            @toggle="toggleSelection(item.appid, $event)"
            @open="openGameDetail(item.appid)"
          />
        </div>
        <p v-else class="muted">No games match your search or filters.</p>
      </template>

      <MemberListStatus
        v-else
        :status="status"
        :retrying="librariesLoading"
        loading-text="Loading your library…"
        failed-text="Couldn't load your library."
        syncing-text="Syncing your library from Steam…"
        empty-text="Your library is empty."
        not-synced-text="Sync your Steam data from the overview page first."
        @retry="retryListLoads"
      />
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

.library-toolbar {
  display: grid;
  grid-template-columns: 1fr repeat(4, auto);
  gap: 0.5rem;
  margin-top: 1rem;
}

.library-count {
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
  .library-toolbar {
    grid-template-columns: 1fr;
  }

  .select {
    min-width: 0;
  }
}
</style>
