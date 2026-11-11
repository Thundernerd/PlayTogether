import type { MemberLibrary, MemberWishlist, SharedGame } from '~/types/api'

export type ListKey = 'libraries' | 'wishlists' | 'shared'

export interface ListData {
  libraries: MemberLibrary[]
  wishlists: MemberWishlist[]
  shared: SharedGame[]
}

/** Everything the app knows about one of the group's lists. */
export interface MemberListState<K extends ListKey = ListKey> {
  data: ListData[K]
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

export function emptyMemberLists(): MemberLists {
  return {
    libraries: { data: [], loading: false, error: null },
    wishlists: { data: [], loading: false, error: null },
    shared: { data: [], loading: false, error: null },
  }
}

/**
 * The shared record of every list, one entry per ListKey. Shared like the rest of the group
 * state, so every page sees the loads the layout starts. useGroup() owns the writes.
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
    /** Counts (nav badge, overview link) show the failed marker instead of a number. */
    failed: computed(() => list.value.error !== null),
  }
}
