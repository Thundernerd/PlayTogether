<script setup lang="ts">
import type { SharedGame } from '~/types/api'
import { headerImageSrc, refetchHeaderImage } from '~/utils/steamImages'

const props = defineProps<{
  item: SharedGame
  disabled?: boolean
}>()

const emit = defineEmits<{
  vote: [vote: 'up' | 'down' | 'none']
  rate: [stars: number | null]
  open: []
}>()

const { api } = useApi()
const imageUrl = ref<string | null>(headerImageSrc(props.item.game.header_image_url))
const imageRetryDone = ref(false)

watch(
  () => props.item.game.header_image_url,
  (url) => {
    imageUrl.value = headerImageSrc(url)
    imageRetryDone.value = false
  },
)

async function onImageError() {
  if (imageRetryDone.value) return
  imageRetryDone.value = true
  try {
    const url = await refetchHeaderImage(
      (appid) => api(`/api/games/${appid}`),
      props.item.appid,
    )
    if (url) {
      imageUrl.value = url
      imageRetryDone.value = false
    }
  } catch {
    imageUrl.value = null
  }
}

const selectedNames = computed(() => props.item.selected_by.map((m) => m.display_name).join(', '))

function setStars(value: number) {
  const next = props.item.my_star_rating === value ? null : value
  emit('rate', next)
}
</script>

<template>
  <article class="shared-row card">
    <button type="button" class="media-button" @click="emit('open')">
      <div v-if="!imageUrl" class="thumb thumb-placeholder" aria-hidden="true" />
      <img
        v-else
        :src="imageUrl"
        :alt="item.game.name"
        class="thumb"
        loading="lazy"
        @error="onImageError"
      />
    </button>
    <div class="content">
      <div class="header">
        <button type="button" class="title-button" @click="emit('open')">
          <h3>{{ item.game.name }}</h3>
        </button>
        <div class="score">
          Score {{ item.net_score }}
          <span v-if="item.avg_stars" class="avg">· ★ {{ item.avg_stars.toFixed(1) }}</span>
        </div>
      </div>
      <GameDiscountBadge :price="item.game.price" />
      <p class="desc">{{ item.game.short_description }}</p>
      <p class="picked">Picked by: {{ selectedNames }}</p>
      <div class="tags">
        <span v-for="tag in item.game.tags.slice(0, 5)" :key="tag" class="tag">{{ tag }}</span>
      </div>
      <div class="actions" @click.stop>
        <div class="votes">
          <button
            class="btn vote"
            :class="{ active: item.my_vote === 1 }"
            :disabled="disabled"
            @click="emit('vote', item.my_vote === 1 ? 'none' : 'up')"
          >
            ▲ {{ item.upvotes }}
          </button>
          <button
            class="btn vote"
            :class="{ active: item.my_vote === -1 }"
            :disabled="disabled"
            @click="emit('vote', item.my_vote === -1 ? 'none' : 'down')"
          >
            ▼ {{ item.downvotes }}
          </button>
        </div>
        <div class="stars">
          <span class="stars-label">Rating</span>
          <button
            v-for="n in 5"
            :key="n"
            class="star"
            :class="{ filled: (item.my_star_rating || 0) >= n }"
            :disabled="disabled"
            @click="setStars(n)"
          >
            ★
          </button>
        </div>
      </div>
    </div>
  </article>
</template>

<style scoped>
.shared-row {
  display: grid;
  grid-template-columns: 180px 1fr;
  gap: 1rem;
  align-items: start;
}

.thumb {
  width: 100%;
  aspect-ratio: 460 / 215;
  object-fit: cover;
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.thumb-placeholder {
  background: linear-gradient(
    110deg,
    var(--surface-2) 8%,
    var(--border) 18%,
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

.media-button,
.title-button {
  appearance: none;
  border: none;
  background: transparent;
  padding: 0;
  color: inherit;
  text-align: left;
  cursor: pointer;
}

.media-button {
  width: 100%;
}

.title-button h3 {
  margin: 0;
  font-size: 1.0625rem;
  /* Keeps the button's user-agent font, as before --font-heading; see #5. */
  font-family: inherit;
}

.title-button:hover h3,
.media-button:hover .thumb {
  opacity: 0.92;
}

.header {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  align-items: baseline;
}

.score {
  color: var(--muted);
  font-size: 0.875rem;
  white-space: nowrap;
}

.avg {
  color: var(--rating-average-text);
}

.desc,
.picked {
  margin: 0.35rem 0 0;
  color: var(--muted);
  font-size: 0.875rem;
}

.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  margin-top: 0.5rem;
}

.tag {
  font-size: 0.6875rem;
  padding: 0.15rem 0.45rem;
  border-radius: var(--radius-pill);
  background: var(--surface-2);
  border: 1px solid var(--border);
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
  margin-top: 0.75rem;
  align-items: center;
}

.votes {
  display: flex;
  gap: 0.5rem;
}

.vote.active {
  border-color: var(--accent);
  color: var(--accent);
}

.stars {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.stars-label {
  font-size: 0.8125rem;
  color: var(--muted);
  margin-right: 0.25rem;
}

.star {
  border: none;
  background: transparent;
  color: var(--rating-empty);
  font-size: 1.125rem;
  cursor: pointer;
  padding: 0 0.1rem;
}

.star.filled {
  color: var(--rating-filled);
}

.star:disabled,
.vote:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

@media (max-width: 720px) {
  .shared-row {
    grid-template-columns: 1fr;
  }
}
</style>
