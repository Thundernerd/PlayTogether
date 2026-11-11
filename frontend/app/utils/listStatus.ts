import type { SyncJobStatus } from '~/types/api'

// What a list page shows, checked in this order. 'items' wins whenever there is something to
// show. 'loading' only covers a list's first load, so poll, vote and rate refreshes keep
// the current text (such as 'syncing' or 'empty') instead of swapping it for a spinner.
export type ListStatus = 'items' | 'failed' | 'loading' | 'syncing' | 'empty' | 'not-synced'

export function listStatus(list: {
  itemCount: number
  /**
   * Whether a load has returned the list yet (for a member's own list, their entry in it;
   * for the shared list, the record's `loaded` marker);
   * once it has, a reload keeps the current status instead of showing 'loading'. Left out,
   * every load shows 'loading'.
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
  // A failed load stays failed while a retry or poll runs, even before the first success.
  if (list.error) return 'failed'
  if (list.loading && !list.loaded) return 'loading'
  if (!list.sync) return 'empty'
  if (list.sync.status === 'running') return 'syncing'
  if (list.sync.complete) return 'empty'
  return 'not-synced'
}
