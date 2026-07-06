<script setup lang="ts">
import type { GamePrice } from '~/types/api'

const props = defineProps<{
  price?: GamePrice | null
  variant?: 'cover' | 'inline'
}>()

const hasDiscount = computed(
  () => (props.price?.discount_percent ?? 0) > 0,
)
</script>

<template>
  <span
    v-if="hasDiscount"
    class="discount-badge"
    :class="variant ?? 'cover'"
    :title="price?.formatted_final ? `Now ${price.formatted_final}` : undefined"
  >
    -{{ price!.discount_percent }}%
  </span>
  <span v-else-if="variant === 'inline' && price?.formatted_final" class="price-inline">
    {{ price.formatted_final }}
  </span>
</template>

<style scoped>
.discount-badge {
  font-weight: 700;
  line-height: 1;
  border-radius: 6px;
  background: #15803d;
  color: #fff;
  white-space: nowrap;
}

.discount-badge.cover {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  font-size: 0.75rem;
  padding: 0.25rem 0.45rem;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35);
}

.discount-badge.inline {
  display: inline-block;
  font-size: 0.6875rem;
  padding: 0.15rem 0.45rem;
  vertical-align: middle;
}

.price-inline {
  font-size: 0.8125rem;
  color: var(--muted);
  white-space: nowrap;
}
</style>
