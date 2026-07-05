<script setup lang="ts">
import type { GameInfo } from '~/types/api'

const props = defineProps<{
  game?: GameInfo | null
  appid: number
  wantsToPlay?: boolean
  showCheckbox?: boolean
  disabled?: boolean
  loading?: boolean
}>()

const emit = defineEmits<{
  toggle: [value: boolean]
}>()

const title = computed(() => {
  if (props.game?.name) return props.game.name
  if (props.loading) return `Loading app ${props.appid}…`
  return `Steam app ${props.appid}`
})

const description = computed(() => {
  if (props.game?.short_description) return props.game.short_description
  if (props.loading) return 'Fetching game details from Steam…'
  return 'Details unavailable for this Steam app.'
})

const image = computed(
  () =>
    props.game?.header_image_url ||
    `https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/${props.appid}/header.jpg`,
)
</script>

<template>
  <article class="game-card">
    <img :src="image" :alt="title" class="cover" loading="lazy" />
    <div class="body">
      <h3 class="title">{{ title }}</h3>
      <p class="desc" :class="{ muted: loading || !game?.short_description }">{{ description }}</p>
      <div v-if="game?.tags?.length" class="tags">
        <span v-for="tag in game.tags.slice(0, 5)" :key="tag" class="tag">{{ tag }}</span>
      </div>
      <label v-if="showCheckbox" class="checkbox-row">
        <input
          type="checkbox"
          :checked="wantsToPlay"
          :disabled="disabled"
          @change="emit('toggle', ($event.target as HTMLInputElement).checked)"
        />
        Want to play together
      </label>
    </div>
  </article>
</template>

<style scoped>
.game-card {
  border: 1px solid var(--border);
  border-radius: 12px;
  overflow: hidden;
  background: var(--bg);
  display: flex;
  flex-direction: column;
}

.cover {
  width: 100%;
  aspect-ratio: 460 / 215;
  object-fit: cover;
  background: var(--surface-2);
}

.body {
  padding: 0.875rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  flex: 1;
}

.title {
  margin: 0;
  font-size: 1rem;
}

.desc {
  margin: 0;
  font-size: 0.875rem;
  color: var(--muted);
  line-height: 1.45;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.muted {
  font-style: italic;
}

.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}

.tag {
  font-size: 0.6875rem;
  padding: 0.15rem 0.45rem;
  border-radius: 999px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: #cbd5e1;
}

.checkbox-row {
  margin-top: auto;
  padding-top: 0.5rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  cursor: pointer;
}

.checkbox-row input {
  width: 1rem;
  height: 1rem;
}
</style>
