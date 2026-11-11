import type { MemberLibrary, MemberWishlist, SharedGame } from '~/types/api'

/** Every list the group page loads, in the order the layout reports their errors. */
export const LIST_KEYS = ['libraries', 'wishlists', 'shared'] as const
export type ListKey = (typeof LIST_KEYS)[number]

export interface ListData {
  libraries: MemberLibrary[]
  wishlists: MemberWishlist[]
  shared: SharedGame[]
}

/** Everything the app knows about one of the group's lists. */
export interface MemberListState<K extends ListKey = ListKey> {
  data: ListData[K]
  /**
   * A load of this list has succeeded since the last reset, so `data` is a real list (maybe
   * an empty one) and an error only means the last refresh failed.
   */
  loaded: boolean
  /** A request for this list is in flight. */
  loading: boolean
  /**
   * The last failed load's message, kept until a load of this list succeeds, so a page can
   * tell "nothing loaded" from "loaded and empty". Not clearing it when a retry or poll
   * starts keeps the failed state steady while a sync keeps polling a failing endpoint.
   */
  error: string | null
}

export type MemberLists = { [K in ListKey]: MemberListState<K> }

function emptyMemberList(): MemberListState {
  return { data: [], loaded: false, loading: false, error: null }
}

export function emptyMemberLists(): MemberLists {
  return Object.fromEntries(LIST_KEYS.map((key) => [key, emptyMemberList()])) as MemberLists
}

/**
 * The shared record of every list, one entry per ListKey. Shared like the rest of the group
 * state, so every page sees the loads the layout starts. useGroup() loads and resets it;
 * useGameDetail() patches game info into the loaded data.
 */
export function useMemberLists() {
  return useState<MemberLists>('groupMemberLists', emptyMemberLists)
}

/** Read-only view of one list, plus what the views derive from it. */
export function useMemberList<K extends ListKey>(key: K) {
  const lists = useMemberLists()
  const list = computed((): MemberListState<K> => lists.value[key])

  return {
    data: computed(() => list.value.data),
    loading: computed(() => list.value.loading),
    error: computed(() => list.value.error),
    loaded: computed(() => list.value.loaded),
    /** What the list's count (nav badge, overview link) shows. */
    countStatus: computed(() => countStatus(list.value)),
    /** The list loaded before, but its last refresh failed: pages show the last list. */
    refreshFailed: computed(() => list.value.loaded && list.value.error !== null),
  }
}
