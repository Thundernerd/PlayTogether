import { describe, expect, it } from 'vitest'
import { countStatus, listStatus } from '~/utils/listStatus'

describe('countStatus (nav badge and overview quick link)', () => {
  it('shows the failed marker when a list that never loaded fails', () => {
    expect(countStatus({ loaded: false, loading: false, error: 'boom' })).toBe('failed')
  })

  it('keeps the last count with a refresh-failed hint when a loaded list fails to refresh', () => {
    expect(countStatus({ loaded: true, loading: false, error: 'boom' })).toBe('stale')
  })

  it('keeps the stale count steady while a refresh of it is in flight', () => {
    expect(countStatus({ loaded: true, loading: true, error: 'boom' })).toBe('stale')
  })

  it('shows a spinner while a list without errors loads, then its count', () => {
    expect(countStatus({ loaded: false, loading: true, error: null })).toBe('loading')
    expect(countStatus({ loaded: true, loading: false, error: null })).toBe('count')
  })
})

describe('listStatus (list page)', () => {
  it('shows the blocking failed state when the list never loaded', () => {
    expect(listStatus({ itemCount: 0, loaded: false, loading: false, error: 'boom' })).toBe('failed')
  })

  it('keeps showing a loaded, empty list when a refresh of it fails', () => {
    expect(listStatus({ itemCount: 0, loaded: true, loading: false, error: 'boom' })).toBe('empty')
  })
})
