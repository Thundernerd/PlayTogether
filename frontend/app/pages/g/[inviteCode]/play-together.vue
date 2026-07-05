<script setup lang="ts">
const group = useGroup()
const { sharedGames, actionLoading, vote, rate, ensureLoaded, groupBasePath } = group

onMounted(() => ensureLoaded({ wishlists: false, shared: true }))
</script>

<template>
  <div class="list-page">
    <section class="card">
      <h2 class="section-title">Play together</h2>
      <p class="section-subtitle">
        Games someone wants to play — vote and rate to help the group decide what to play next.
      </p>

      <div v-if="sharedGames.length" class="shared-list">
        <SharedGameRow
          v-for="game in sharedGames"
          :key="game.appid"
          :item="game"
          :disabled="actionLoading"
          @vote="vote(game.appid, $event)"
          @rate="rate(game.appid, $event)"
        />
      </div>
      <p v-else class="muted">
        No games selected yet. Mark games on
        <NuxtLink :to="`${groupBasePath}/wishlist`">your wishlist</NuxtLink>.
      </p>
    </section>

    <ScrollToTop />
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

.muted {
  color: var(--muted);
}
</style>
