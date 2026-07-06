<script setup lang="ts">
const props = defineProps<{
  basePath: string
  wishlistCount?: number
  libraryCount?: number
  sharedCount?: number
}>()

const route = useRoute()

const links = computed(() => [
  { to: props.basePath, label: 'Overview', exact: true },
  { to: `${props.basePath}/wishlist`, label: 'My wishlist', count: props.wishlistCount },
  { to: `${props.basePath}/library`, label: 'My library', count: props.libraryCount },
  { to: `${props.basePath}/play-together`, label: 'Play together', count: props.sharedCount },
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
      <span v-if="link.count != null" class="count">{{ link.count }}</span>
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
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--muted);
  font-size: 0.875rem;
  text-decoration: none;
  transition: border-color 0.15s, color 0.15s, background 0.15s;
}

.nav-link:hover {
  border-color: var(--accent);
  color: var(--text);
}

.nav-link.active {
  border-color: rgba(56, 189, 248, 0.55);
  background: rgba(56, 189, 248, 0.12);
  color: #e0f2fe;
}

.count {
  font-size: 0.75rem;
  padding: 0.05rem 0.4rem;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--text);
}
</style>
