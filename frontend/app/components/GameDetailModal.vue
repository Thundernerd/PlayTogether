<script setup lang="ts">
import { renderGameDescription } from '~/utils/bbcode'
import { headerImageSrc, resolveApiMediaUrl } from '~/utils/steamImages'

const config = useRuntimeConfig()
const { detail, loading, error, isOpen, close, retry } = useGameDetail()

type MediaItem =
  | { kind: 'image'; label: string; url: string; thumb: string }
  | { kind: 'video'; label: string; url: string; webmUrl?: string | null; thumb: string }

const selectedIndex = ref(0)
const videoRef = ref<HTMLVideoElement | null>(null)

const descriptionHtml = computed(() =>
  renderGameDescription(detail.value?.full_description || detail.value?.short_description),
)

const releaseLabel = computed(() => {
  const released = detail.value?.is_released
  if (released === true) return 'Released'
  if (released === false) return 'Unreleased'
  return 'Release unknown'
})

const mediaItems = computed<MediaItem[]>(() => {
  const game = detail.value
  if (!game) return []

  const apiBase = config.public.apiBase
  const headerUrl = headerImageSrc(game.header_image_url)

  const items: MediaItem[] = []
  if (headerUrl) {
    items.push({
      kind: 'image',
      label: 'Header',
      url: headerUrl,
      thumb: headerUrl,
    })
  }

  for (const [index, url] of game.screenshots.entries()) {
    items.push({
      kind: 'image',
      label: `Screenshot ${index + 1}`,
      url,
      thumb: url,
    })
  }

  for (const trailer of game.trailers) {
    items.push({
      kind: 'video',
      label: trailer.name,
      url: resolveApiMediaUrl(trailer.video_url, apiBase),
      webmUrl: trailer.video_webm_url
        ? resolveApiMediaUrl(trailer.video_webm_url, apiBase)
        : null,
      thumb: headerImageSrc(trailer.thumbnail_url) || headerUrl || '',
    })
  }

  return items
})

const selectedMedia = computed(() => mediaItems.value[selectedIndex.value] ?? null)

watch(
  () => detail.value?.appid,
  () => {
    selectedIndex.value = 0
  },
)

watch(selectedMedia, async () => {
  await nextTick()
  if (videoRef.value) {
    videoRef.value.load()
  }
})

function selectMedia(index: number) {
  selectedIndex.value = index
}

function onBackdropClick(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    close()
  }
}

watch(isOpen, (open) => {
  if (typeof document === 'undefined') return
  document.body.style.overflow = open ? 'hidden' : ''
})

function onWindowKeydown(event: KeyboardEvent) {
  if (!isOpen.value) return

  if (event.key === 'Escape') {
    close()
    return
  }

  if (mediaItems.value.length === 0) return

  if (event.key === 'ArrowLeft') {
    selectedIndex.value = Math.max(0, selectedIndex.value - 1)
  }

  if (event.key === 'ArrowRight') {
    selectedIndex.value = Math.min(mediaItems.value.length - 1, selectedIndex.value + 1)
  }
}

onMounted(() => {
  window.addEventListener('keydown', onWindowKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onWindowKeydown)
  if (typeof document !== 'undefined') {
    document.body.style.overflow = ''
  }
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="isOpen"
      class="modal-backdrop"
      role="presentation"
      @click="onBackdropClick"
    >
      <div
        class="modal-panel"
        role="dialog"
        aria-modal="true"
        :aria-label="detail?.name || 'Game details'"
      >
        <button type="button" class="close-btn" aria-label="Close" @click="close">×</button>

        <div v-if="loading && !detail" class="modal-state">
          <p>Loading game details…</p>
        </div>

        <div v-else-if="error && !detail" class="modal-state">
          <p>{{ error }}</p>
          <button type="button" class="btn" @click="retry">Retry</button>
        </div>

        <template v-else-if="detail">
          <section class="media-carousel">
            <div class="media-stage">
              <video
                v-if="selectedMedia?.kind === 'video'"
                ref="videoRef"
                :key="`${detail.appid}-${selectedMedia.url}`"
                class="media-main"
                controls
                playsinline
                preload="metadata"
                :poster="selectedMedia.thumb"
              >
                <source :src="selectedMedia.url" type="video/mp4" />
                <source v-if="selectedMedia.webmUrl" :src="selectedMedia.webmUrl" type="video/webm" />
              </video>
              <img
                v-else-if="selectedMedia"
                :key="selectedMedia.url"
                :src="selectedMedia.url"
                :alt="selectedMedia.label"
                class="media-main"
              />
            </div>

            <div class="media-thumbs" role="tablist" aria-label="Game media">
              <button
                v-for="(item, index) in mediaItems"
                :key="`${detail.appid}-${item.kind}-${index}`"
                type="button"
                class="media-thumb"
                :class="{ active: index === selectedIndex }"
                role="tab"
                :aria-selected="index === selectedIndex"
                :aria-label="item.label"
                @click="selectMedia(index)"
              >
                <img :src="item.thumb" :alt="item.label" loading="lazy" />
                <span v-if="item.kind === 'video'" class="play-badge">▶</span>
              </button>
            </div>
          </section>

          <div class="content">
            <div class="header">
              <div class="header-main">
                <h2 class="title">{{ detail.name }}</h2>
                <div class="meta-badges">
                  <span class="release-badge">{{ releaseLabel }}</span>
                  <GameDiscountBadge v-if="detail.price" :price="detail.price" />
                </div>
                <div v-if="detail.tags.length" class="tags">
                  <span v-for="tag in detail.tags" :key="tag" class="tag">{{ tag }}</span>
                </div>
              </div>
              <a :href="detail.store_url" target="_blank" rel="noopener noreferrer" class="store-link">
                View on Steam
              </a>
            </div>

            <section v-if="descriptionHtml" class="section">
              <h3>About</h3>
              <div class="description" v-html="descriptionHtml" />
            </section>
          </div>
        </template>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: var(--overlay-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1.5rem;
}

.modal-panel {
  position: relative;
  width: min(920px, 100%);
  max-height: min(90vh, 960px);
  overflow: auto;
  border-radius: var(--radius-sm);
  border: var(--line-heavy);
  background: var(--surface);
  box-shadow: var(--shadow-modal);
}

.close-btn {
  position: sticky;
  top: 0.75rem;
  float: right;
  margin: 0.75rem 0.75rem 0 0;
  z-index: 2;
  width: 2rem;
  height: 2rem;
  border-radius: var(--radius-sm);
  border: var(--line);
  background: var(--surface);
  box-shadow: var(--shadow-sm);
  color: var(--text);
  font-size: 1.25rem;
  line-height: 1;
  cursor: pointer;
}

.media-carousel {
  background: var(--media-letterbox);
}

.media-stage {
  width: 100%;
  background: var(--media-letterbox);
}

.media-main {
  display: block;
  width: 100%;
  aspect-ratio: 460 / 215;
  object-fit: contain;
  background: var(--media-letterbox);
}

.media-thumbs {
  display: flex;
  gap: 0.5rem;
  overflow-x: auto;
  padding: 0.75rem;
  background: var(--surface-2);
  border-top: var(--line);
}

.media-thumb {
  position: relative;
  flex: 0 0 auto;
  width: 112px;
  height: 63px;
  padding: 0;
  border-radius: var(--radius-xs);
  border: 2px solid transparent;
  background: transparent;
  cursor: pointer;
  overflow: hidden;
}

.media-thumb.active {
  border-color: var(--accent);
}

.media-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.play-badge {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  background: var(--media-scrim);
  color: var(--on-media);
  font-size: 1.25rem;
  pointer-events: none;
}

.content {
  padding: 1.25rem 1.5rem 1.5rem;
}

.header {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  align-items: start;
}

.header-main {
  min-width: 0;
}

.title {
  margin: 0;
  font-size: 1.5rem;
}

.meta-badges {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.75rem;
  margin-top: 0.35rem;
}

.meta-badges :deep(.price-line) {
  font-size: 0.875rem;
}

.release-badge {
  display: inline-block;
  font-size: 0.75rem;
  padding: 0.15rem 0.5rem;
  border-radius: var(--radius-xs);
  background: transparent;
  border: var(--line-thin);
  color: var(--text);
}

.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  margin-top: 0.75rem;
}

.tag {
  font-size: 0.75rem;
  padding: 0.2rem 0.55rem;
}

.store-link {
  white-space: nowrap;
  font-size: 0.875rem;
}

.section {
  margin-top: 1.25rem;
}

.section h3 {
  margin: 0 0 0.5rem;
  font-size: 1.25rem;
  font-weight: 800;
  color: var(--accent);
}

.description {
  line-height: 1.6;
  color: var(--text);
}

.description :deep(p) {
  margin: 0 0 0.75rem;
}

.description :deep(ul),
.description :deep(ol) {
  margin: 0 0 0.75rem;
  padding-left: 1.25rem;
}

.description :deep(img) {
  max-width: 100%;
  border-radius: var(--radius-xs);
}

.modal-state {
  padding: 2.5rem 1.5rem;
  text-align: center;
  color: var(--muted);
}

@media (max-width: 720px) {
  .modal-backdrop {
    padding: 0.75rem;
  }

  .header {
    flex-direction: column;
  }

  .media-thumb {
    width: 88px;
    height: 50px;
  }
}
</style>
