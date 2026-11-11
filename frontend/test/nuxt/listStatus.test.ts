import { describe, expect, it } from 'vitest'
import { listStatus } from '~/utils/listStatus'

describe('listStatus', () => {
  it('shows loading during the first load of an empty list', () => {
    expect(listStatus({ itemCount: 0, loaded: false, loading: true, error: null })).toBe('loading')
  })

  it('keeps an already loaded empty list empty while it reloads', () => {
    expect(listStatus({ itemCount: 0, loaded: true, loading: true, error: null })).toBe('empty')
  })

  it('keeps the failed state while a loaded list reloads after a failure', () => {
    expect(listStatus({ itemCount: 0, loaded: true, loading: true, error: 'boom' })).toBe('failed')
  })
})
