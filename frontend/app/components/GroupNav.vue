<script setup lang="ts">
import type { CountStatus } from '~/utils/listStatus'

// A list that never loaded shows a dash, not a count that reads as an empty list; one whose
// last refresh failed keeps its last count with a "couldn't refresh" mark.
const props = defineProps<{
  basePath: string
  wishlistCount?: number
  libraryCount?: number
  sharedCount?: number
  libraryStatus?: CountStatus
  wishlistStatus?: CountStatus
  sharedStatus?: CountStatus
}>()

const route = useRoute()

const links = computed(() => [
  { to: props.basePath, label: 'Overview', exact: true },
  {
    to: `${props.basePath}/library`,
    label: 'My library',
    count: props.libraryCount,
    status: props.libraryStatus,
  },
  {
    to: `${props.basePath}/wishlist`,
    label: 'My wishlist',
    count: props.wishlistCount,
    status: props.wishlistStatus,
  },
  {
    to: `${props.basePath}/play-together`,
    label: 'Play together',
    count: props.sharedCount,
    status: props.sharedStatus,
  },
])

function isActive(link: { to: string; exact?: boolean }) {
  if (link.exact) {
    return route.path === link.to || route.path === `${link.to}/`
  }
  return route.path.startsWith(link.to)
}
</script>

<template>
  <nav class="group-nav">
    <NuxtLink
      v-for="link in links"
      :key="link.to"
      :to="link.to"
      class="nav-link"
      :class="{ active: isActive(link) }"
    >
      {{ link.label }}
      <span
        v-if="link.status === 'failed'"
        class="count failed"
        role="img"
        title="Couldn't load"
        aria-label="Couldn't load"
      >–</span>
      <span
        v-else-if="link.status === 'stale' && link.count != null"
        class="count stale"
        role="img"
        title="Couldn't refresh: showing the last count"
        :aria-label="`${link.count}, couldn't refresh`"
      >
        {{ link.count }}<span class="stale-mark" aria-hidden="true">!</span>
      </span>
      <span v-else-if="link.count != null" class="count">
        <LoadingSpinner v-if="link.status === 'loading'" size="sm" />
        <template v-else>{{ link.count }}</template>
      </span>
    </NuxtLink>
  </nav>
</template>

<style scoped>
.group-nav {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.nav-link {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.45rem 0.85rem;
  border-radius: var(--radius-sm);
  border: var(--line);
  background: var(--surface);
  color: var(--text);
  font-size: 0.875rem;
  font-weight: 700;
  text-decoration: none;
  transition: color 0.15s, background 0.15s;
}

.nav-link:hover {
  background: var(--highlight);
}

.nav-link.active {
  background: var(--selected-bg);
  color: var(--selected-text);
}

.count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 1.25rem;
  min-height: 1.1rem;
  font-size: 0.75rem;
  padding: 0.05rem 0.4rem;
  border-radius: var(--radius-pill);
  background: var(--surface-2);
  color: var(--text);
}

.count.failed {
  color: var(--danger-text);
}

.count.stale {
  gap: 0.15rem;
}

.stale-mark {
  color: var(--warning-text);
  font-weight: 800;
}

.nav-link.active .count {
  background: var(--highlight);
}
</style>
