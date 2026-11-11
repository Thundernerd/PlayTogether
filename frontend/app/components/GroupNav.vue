<script setup lang="ts">
const props = defineProps<{
  basePath: string
  wishlistCount?: number
  libraryCount?: number
  sharedCount?: number
  libraryLoading?: boolean
  wishlistLoading?: boolean
  sharedLoading?: boolean
}>()

const route = useRoute()

const links = computed(() => [
  { to: props.basePath, label: 'Overview', exact: true },
  {
    to: `${props.basePath}/library`,
    label: 'My library',
    count: props.libraryCount,
    loading: props.libraryLoading,
  },
  {
    to: `${props.basePath}/wishlist`,
    label: 'My wishlist',
    count: props.wishlistCount,
    loading: props.wishlistLoading,
  },
  {
    to: `${props.basePath}/play-together`,
    label: 'Play together',
    count: props.sharedCount,
    loading: props.sharedLoading,
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
      <span v-if="link.count != null" class="count">
        <LoadingSpinner v-if="link.loading" size="sm" />
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

.nav-link.active .count {
  background: var(--highlight);
}
</style>
