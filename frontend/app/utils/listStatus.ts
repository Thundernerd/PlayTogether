import type { SyncJobStatus } from '~/types/api'

// What a list page shows, checked in this order. 'items' wins whenever there is something to
// show. For a member's own list, 'loading' only covers the first load, so poll refreshes
// during a sync keep the 'syncing' text instead of swapping it for a spinner. Likewise
// 'failed' only covers a list that never loaded: a failed refresh of a loaded list keeps its
// status, and the page adds a non-blocking "couldn't refresh" line (countStatus 'stale').
export type ListStatus = 'items' | 'loading' | 'failed' | 'syncing' | 'empty' | 'not-synced'

export function listStatus(list: {
  itemCount: number
  /**
   * Whether a load has returned the list yet (for a member's own list, their entry in it);
   * once it has, a reload keeps the current status instead of showing 'loading', and a failed
   * reload keeps it instead of showing 'failed'. Left out, every load shows 'loading' and
   * every error 'failed'.
   */
  loaded?: boolean
  loading: boolean
  error: string | null
  /** The member's Steam sync, for lists that come from it. Left out, no items is 'empty'. */
  sync?: {
    status: SyncJobStatus | undefined
    complete: boolean
  }
}): ListStatus {
  if (list.itemCount > 0) return 'items'
  if (list.loading && !list.loaded) return 'loading'
  if (list.error && !list.loaded) return 'failed'
  if (!list.sync) return 'empty'
  if (list.sync.status === 'running') return 'syncing'
  if (list.sync.complete) return 'empty'
  return 'not-synced'
}

// What a list's count (nav badge, overview quick link) shows, checked in this order.
// 'failed' (a marker instead of a number) only covers a list that never loaded; once it has,
// a failed refresh is 'stale': the last count plus a "couldn't refresh" hint. Both are checked
// before 'loading' so they hold steady while a sync keeps polling a failing endpoint.
export type CountStatus = 'failed' | 'stale' | 'loading' | 'count'

export function countStatus(list: {
  /** Whether a load of this list has ever succeeded. */
  loaded: boolean
  loading: boolean
  error: string | null
}): CountStatus {
  if (list.error) return list.loaded ? 'stale' : 'failed'
  if (list.loading) return 'loading'
  return 'count'
}
