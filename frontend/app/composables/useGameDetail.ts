import type { GameDetail, GameInfo } from '~/types/api'
import type { MemberLists } from '~/composables/useMemberList'
import type { Ref } from 'vue'

function patchGameInfoInLists(lists: Ref<MemberLists>, detail: GameDetail) {

  const patch = (game: GameInfo) => {
    if (game.appid !== detail.appid) return game
    return {
      ...game,
      header_image_url: detail.header_image_url,
      name: detail.name,
      short_description: detail.short_description,
      tags: detail.tags.length ? detail.tags : game.tags,
      is_released: detail.is_released ?? game.is_released,
      price: detail.price ?? game.price,
      play_modes: detail.play_modes ?? game.play_modes,
    }
  }

  lists.value.wishlists.data = lists.value.wishlists.data.map((wishlist) => ({
    ...wishlist,
    items: wishlist.items.map((item) =>
      item.game ? { ...item, game: patch(item.game) } : item,
    ),
  }))

  lists.value.libraries.data = lists.value.libraries.data.map((library) => ({
    ...library,
    items: library.items.map((item) =>
      item.game ? { ...item, game: patch(item.game) } : item,
    ),
  }))

  lists.value.shared.data = lists.value.shared.data.map((item) =>
    item.appid === detail.appid ? { ...item, game: patch(item.game) } : item,
  )
}

export function useGameDetail() {
  const { api } = useApi()
  // Read in setup: fetchDetail patches the lists after an await, outside the setup context.
  const lists = useMemberLists()
  const activeAppid = useState<number | null>('game-detail-appid', () => null)
  const cache = useState<Record<number, GameDetail>>('game-detail-cache-v3', () => ({}))
  const loading = ref(false)
  const error = ref<string | null>(null)

  const detail = computed(() => {
    const appid = activeAppid.value
    if (appid == null) return null
    return cache.value[appid] ?? null
  })

  const isOpen = computed(() => activeAppid.value != null)

  async function fetchDetail(appid: number, force = false) {
    if (!force && cache.value[appid]) {
      return cache.value[appid]
    }

    loading.value = true
    error.value = null

    try {
      const result = await api<GameDetail>(`/api/games/${appid}`)
      cache.value = { ...cache.value, [appid]: result }
      patchGameInfoInLists(lists, result)
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load game details'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function open(appid: number) {
    activeAppid.value = appid

    if (cache.value[appid]) {
      fetchDetail(appid, true).catch(() => undefined)
      return
    }

    await fetchDetail(appid).catch(() => undefined)
  }

  function close() {
    activeAppid.value = null
    error.value = null
    loading.value = false
  }

  async function retry() {
    const appid = activeAppid.value
    if (appid == null) return
    await fetchDetail(appid, true).catch(() => undefined)
  }

  return {
    activeAppid,
    detail,
    loading,
    error,
    isOpen,
    open,
    close,
    retry,
  }
}
