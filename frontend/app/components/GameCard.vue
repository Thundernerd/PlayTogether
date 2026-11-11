<script setup lang="ts">
import type { GameInfo } from '~/types/api'
import { headerImageSrc, refetchHeaderImage } from '~/utils/steamImages'

const props = defineProps<{
  game?: GameInfo | null
  appid: number
  wantsToPlay?: boolean
  showCheckbox?: boolean
  disabled?: boolean
  loading?: boolean
  subtitle?: string
}>()

const emit = defineEmits<{
  toggle: [value: boolean]
  open: []
}>()

const { api } = useApi()
const imageUrl = ref<string | null>(null)
const imageRetryDone = ref(false)

watch(
  () => props.game?.header_image_url,
  () => {
    imageUrl.value = headerImageSrc(props.game?.header_image_url)
    imageRetryDone.value = false
  },
  { immediate: true },
)

function openDetails() {
  if (props.loading) return
  emit('open')
}

function onCardKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault()
    openDetails()
  }
}

async function onImageError() {
  if (imageRetryDone.value || props.loading) return
  imageRetryDone.value = true
  try {
    const url = await refetchHeaderImage(
      (appid) => api(`/api/games/${appid}`),
      props.appid,
    )
    if (url) {
      imageUrl.value = url
      imageRetryDone.value = false
    }
  } catch {
    imageUrl.value = null
  }
}

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

const showImagePlaceholder = computed(
  () => props.loading || !imageUrl.value,
)
</script>

<template>
  <article
    class="game-card"
    :class="{ clickable: !loading }"
    :role="loading ? undefined : 'button'"
    :tabindex="loading ? -1 : 0"
    @click="openDetails"
    @keydown="onCardKeydown"
  >
    <div v-if="showImagePlaceholder" class="cover cover-placeholder" aria-hidden="true" />
    <img
      v-else
      :src="imageUrl!"
      :alt="title"
      class="cover"
      loading="lazy"
      @error="onImageError"
    />
    <div class="body">
      <h3 class="title">{{ title }}</h3>
      <p v-if="subtitle" class="subtitle">{{ subtitle }}</p>
      <GameDiscountBadge :price="game?.price" />
      <p class="desc" :class="{ muted: loading || !game?.short_description }">{{ description }}</p>
      <div v-if="game?.tags?.length" class="tags">
        <span v-for="tag in game.tags.slice(0, 5)" :key="tag" class="tag">{{ tag }}</span>
      </div>
      <label v-if="showCheckbox" class="checkbox-row" @click.stop>
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
  border: var(--line);
  border-radius: var(--radius-sm);
  overflow: hidden;
  background: var(--surface-bright);
  box-shadow: var(--shadow-tile);
  display: flex;
  flex-direction: column;
  transition: transform 0.12s, box-shadow 0.12s;
}

.game-card.clickable {
  cursor: pointer;
}

.game-card.clickable:hover {
  transform: translate(-2px, -2px) rotate(-0.4deg);
  box-shadow: var(--shadow-tile-hover);
}

.game-card.clickable:focus-visible {
  outline: var(--focus-outline);
  outline-offset: 3px;
}

@media (prefers-reduced-motion: reduce) {
  .game-card {
    transition: none;
  }
}

.cover {
  width: 100%;
  aspect-ratio: 460 / 215;
  object-fit: cover;
  background: var(--surface-2);
  border-bottom: var(--line);
}

.cover-placeholder {
  background: linear-gradient(
    110deg,
    var(--surface-2) 8%,
    var(--surface) 18%,
    var(--surface-2) 33%
  );
  background-size: 200% 100%;
  animation: shimmer 1.4s ease infinite;
}

@keyframes shimmer {
  to {
    background-position-x: -200%;
  }
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
  font-size: 1.125rem;
}

.subtitle {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--muted);
  font-style: italic;
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
