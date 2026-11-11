import type { SyncJobStatus } from '~/types/api'

// What a member's list page shows, checked in this order. 'items' wins whenever there is
// something to show; 'loading' only covers the first load, so poll refreshes during a sync
// keep the 'syncing' text instead of swapping it for a spinner.
export type ListStatus = 'items' | 'loading' | 'failed' | 'syncing' | 'empty' | 'not-synced'

export function memberListStatus(list: {
  itemCount: number
  /** Whether a load has returned the member's entry yet. */
  loaded: boolean
  loading: boolean
  error: string | null
  syncStatus: SyncJobStatus | undefined
  syncComplete: boolean
}): ListStatus {
  if (list.itemCount > 0) return 'items'
  if (list.loading && !list.loaded) return 'loading'
  if (list.error) return 'failed'
  if (list.syncStatus === 'running') return 'syncing'
  if (list.syncComplete) return 'empty'
  return 'not-synced'
}
