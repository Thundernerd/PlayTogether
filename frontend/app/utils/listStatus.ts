import type { SyncJobStatus } from '~/types/api'

// What a list page shows, checked in this order. 'items' wins whenever there is something to
// show. For a member's own list, 'loading' only covers the first load, so poll refreshes
// during a sync keep the 'syncing' text instead of swapping it for a spinner.
export type ListStatus = 'items' | 'loading' | 'failed' | 'syncing' | 'empty' | 'not-synced'

export function memberListStatus(list: {
  itemCount: number
  /**
   * Whether a load has returned the member's entry yet; once it has, a reload keeps the
   * current status instead of showing 'loading'. Left out, every load shows 'loading'.
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
  if (list.error) return 'failed'
  if (!list.sync) return 'empty'
  if (list.sync.status === 'running') return 'syncing'
  if (list.sync.complete) return 'empty'
  return 'not-synced'
}
