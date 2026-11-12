import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { MeResponse } from '~/types/api'

// useApi() calls the global $fetch with the API base prefixed; stub it there.
const api = vi.fn()
vi.stubGlobal('$fetch', api)

const GROUP_ID = 'group-1'
const librariesPath = `/api/groups/${GROUP_ID}/libraries`
const isLibraries = (url: string) => url.endsWith(librariesPath)

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

function librariesCalls() {
  return api.mock.calls.filter(([url]) => isLibraries(url)).length
}

describe('useGroup retryListLoads (Try again)', () => {
  beforeEach(() => {
    api.mockReset()
    api.mockResolvedValue([])
    useState<MeResponse | null>('groupMe').value = {
      group: { id: GROUP_ID, invite_code: 'ABC' },
      member: { id: 'member-1' },
    } as unknown as MeResponse
    useMemberLists().value = emptyMemberLists()
  })

  it('reloads a list once the in-flight request it joined fails, so the click is not lost', async () => {
    const doomed = deferred<never>()
    let librariesRequests = 0
    api.mockImplementation((url: string) =>
      isLibraries(url) && ++librariesRequests === 1 ? doomed.promise : Promise.resolve([]),
    )
    const group = useGroup()

    const poll = group.refreshMemberData()
    const retry = group.retryListLoads()
    doomed.reject(new Error('backend down'))
    await poll
    await retry

    expect(librariesCalls()).toBe(2)
    const libraries = useMemberLists().value.libraries
    expect(libraries.loaded).toBe(true)
    expect(libraries.error).toBeNull()
  })

  it('waits for a joined list request even when the group reload fails first', async () => {
    const doomed = deferred<never>()
    let librariesRequests = 0
    api.mockImplementation((url: string) => {
      if (url.includes('/api/groups/by-invite/')) return Promise.reject(new Error('backend down'))
      if (isLibraries(url)) return ++librariesRequests === 1 ? doomed.promise : Promise.resolve([])
      return Promise.resolve([])
    })
    const group = useGroup()

    const poll = group.refreshMemberData().catch(() => {})
    const retry = group.retryListLoads()
    await poll
    doomed.reject(new Error('backend down'))
    await retry

    expect(librariesRequests).toBe(2)
    expect(useMemberLists().value.libraries.error).toBeNull()
  })

  it('does not send a second request when the in-flight request it joined succeeds', async () => {
    const pending = deferred<[]>()
    api.mockImplementation((url: string) =>
      isLibraries(url) ? pending.promise : Promise.resolve([]),
    )
    const group = useGroup()

    const poll = group.refreshMemberData()
    const retry = group.retryListLoads()
    pending.resolve([])
    await poll
    await retry

    expect(librariesCalls()).toBe(1)
    expect(useMemberLists().value.libraries.loaded).toBe(true)
  })

  it('sends one request per list when nothing is in flight, even if it fails', async () => {
    api.mockImplementation((url: string) =>
      isLibraries(url) ? Promise.reject(new Error('backend down')) : Promise.resolve([]),
    )

    await useGroup().retryListLoads()

    expect(librariesCalls()).toBe(1)
    expect(useMemberLists().value.libraries.error).toBe('backend down')
  })

  it('writes nothing and reloads nothing when the member leaves while the joined request fails', async () => {
    vi.stubGlobal('confirm', () => true)
    useState('groupDetail').value = { group: { name: 'G' }, members: [{}] }
    const doomed = deferred<never>()
    api.mockImplementation((url: string) =>
      isLibraries(url) ? doomed.promise : Promise.resolve([]),
    )
    const group = useGroup()

    const poll = group.refreshMemberData()
    const retry = group.retryListLoads()
    await group.leaveGroup()
    doomed.reject(new Error('backend down'))
    await poll
    await retry

    expect(librariesCalls()).toBe(1)
    expect(useMemberLists().value.libraries.error).toBeNull()
  })
})
