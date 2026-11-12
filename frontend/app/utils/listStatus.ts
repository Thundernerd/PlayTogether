import type { SyncJobStatus } from '~/types/api'

// What a list page shows, checked in this order. 'items' wins whenever there is something to
// show. 'failed' only covers a list that never loaded, and holds steady while a retry or
// poll of it runs: a failed refresh of a loaded list keeps its status, and the page adds a
// non-blocking "couldn't refresh" line (countStatus 'stale'). Likewise 'loading' only covers
// a list's first load, so poll, vote and rate refreshes keep the current text (such as
// 'syncing' or 'empty') instead of swapping it for a spinner.
export type ListStatus = 'items' | 'failed' | 'loading' | 'syncing' | 'empty' | 'not-synced'

export function listStatus(list: {
  itemCount: number
  /**
   * Whether a load of the list has succeeded yet: the list-level flag, so the page agrees
   * with the nav and overview counts even when the member's own entry is missing. Once it
   * has, a reload keeps the current status instead of showing 'loading', and a failed reload
   * keeps it instead of showing 'failed'.
   */
  loaded: boolean
  loading: boolean
  error: string | null
  /** The member's Steam sync, for lists that come from it. Left out, no items is 'empty'. */
  sync?: {
    status: SyncJobStatus | undefined
    complete: boolean
  }
}): ListStatus {
  if (list.itemCount > 0) return 'items'
  if (list.error && !list.loaded) return 'failed'
  if (list.loading && !list.loaded) return 'loading'
  if (!list.sync) return 'empty'
  if (list.sync.status === 'running') return 'syncing'
  if (list.sync.complete) return 'empty'
  return 'not-synced'
}

// What a list's count (nav badge, overview quick link) shows, checked in this order.
// 'failed' (a marker instead of a number) only covers a list that never loaded; once it has,
// a failed refresh is 'stale': the last count plus a "couldn't refresh" hint. Both are checked
// before 'loading' so they hold steady while a sync keeps polling a failing endpoint. 'loading'
// (a spinner) only covers a list's first load, so poll, vote, rate and selection refreshes
// keep the current count instead of flashing a spinner.
export type CountStatus = 'failed' | 'stale' | 'loading' | 'count'

export function countStatus(list: {
  /** Whether a load of this list has ever succeeded. */
  loaded: boolean
  loading: boolean
  error: string | null
}): CountStatus {
  if (list.error) return list.loaded ? 'stale' : 'failed'
  if (list.loading && !list.loaded) return 'loading'
  return 'count'
}
