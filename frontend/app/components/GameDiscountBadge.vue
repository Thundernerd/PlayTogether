<script setup lang="ts">
import type { GamePrice } from '~/types/api'

const props = defineProps<{
  price?: GamePrice | null
}>()

const hasDiscount = computed(
  () => (props.price?.discount_percent ?? 0) > 0,
)
</script>

<template>
  <p v-if="price?.formatted_final" class="price-line">
    <span v-if="hasDiscount && price.formatted_initial" class="was">
      {{ price.formatted_initial }}
    </span>
    <span class="now">{{ price.formatted_final }}</span>
    <span v-if="hasDiscount" class="discount">-{{ price.discount_percent }}%</span>
  </p>
</template>

<style scoped>
.price-line {
  margin: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0.4rem;
  font-size: 0.8125rem;
  line-height: 1.3;
}

.was {
  color: var(--muted);
  text-decoration: line-through;
}

.now {
  color: var(--text);
  font-weight: 600;
}

.discount {
  font-size: 0.6875rem;
  font-weight: 700;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  background: #166534;
  color: #dcfce7;
}
</style>
