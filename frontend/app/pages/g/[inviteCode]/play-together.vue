<script setup lang="ts">
// This page shows its own failed state, so the layout skips the raw list error line.
definePageMeta({ showsListErrors: true })

const group = useGroup()
const { actionLoading, vote, rate, groupBasePath, retryListLoads } = group
const {
  data: sharedGames,
  loaded: sharedLoaded,
  loading: sharedLoading,
  error: sharedError,
  refreshFailed: sharedRefreshFailed,
} = useMemberList('shared')
const { open: openGameDetail } = useGameDetail()

// No sync behind this list, so no items once it has loaded is 'empty'.
const status = computed(() =>
  listStatus({
    itemCount: sharedGames.value.length,
    loaded: sharedLoaded.value,
    loading: sharedLoading.value,
    error: sharedError.value,
  }),
)
</script>

<template>
  <div class="list-page">
    <section class="card">
      <h2 class="section-title">Play together</h2>
      <p class="section-subtitle">
        Games someone wants to play — vote and rate to help the group decide what to play next.
      </p>

      <ListRefreshFailed v-if="sharedRefreshFailed" :retrying="sharedLoading" @retry="retryListLoads" />

      <div v-if="status === 'items'" class="shared-list">
        <SharedGameRow
          v-for="game in sharedGames"
          :key="game.appid"
          :item="game"
          :disabled="actionLoading"
          @vote="vote(game.appid, $event)"
          @rate="rate(game.appid, $event)"
          @open="openGameDetail(game.appid)"
        />
      </div>
      <MemberListStatus
        v-else
        :status="status"
        :retrying="sharedLoading"
        loading-text="Loading shared games…"
        failed-text="Couldn't load the shared games."
        @retry="retryListLoads"
      >
        <template #empty>
          No games selected yet. Mark games on
          <NuxtLink :to="`${groupBasePath}/library`">your library</NuxtLink>
          or
          <NuxtLink :to="`${groupBasePath}/wishlist`">your wishlist</NuxtLink>.
        </template>
      </MemberListStatus>
    </section>

    <ScrollToTop />
    <GameDetailModal />
  </div>
</template>

<style scoped>
.list-page {
  padding-bottom: 5rem;
}

.shared-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin-top: 1rem;
}
</style>
