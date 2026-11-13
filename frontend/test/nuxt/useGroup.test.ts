import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { MeResponse } from '~/types/api'

// useApi() calls the global $fetch with the API base prefixed; stub it there.
const api = vi.fn()
vi.stubGlobal('$fetch', api)

const GROUP_ID = 'group-1'
const librariesPath = `/api/groups/${GROUP_ID}/libraries`
const isLibraries = (url: string) => url.endsWith(librariesPath)
const isGroup = (url: string) => url.includes('/api/groups/by-invite/')

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

// A group the member is about to leave (or has left).
const leftGroup = { group: { name: 'Left group' }, members: [{}] }

// Holds the group request open; every other request resolves at once.
function holdGroupRequest() {
  const pendingGroup = deferred<typeof leftGroup>()
  api.mockImplementation((url: string) =>
    isGroup(url) ? pendingGroup.promise : Promise.resolve([]),
  )
  return pendingGroup
}

function librariesCalls() {
  return api.mock.calls.filter(([url]) => isLibraries(url)).length
}

function groupCalls() {
  return api.mock.calls.filter(([url]) => isGroup(url)).length
}

function signInAsMember() {
  useState<MeResponse | null>('groupMe').value = {
    group: { id: GROUP_ID, invite_code: 'ABC' },
    member: { id: 'member-1' },
  } as unknown as MeResponse
}

describe('useGroup retryListLoads (Try again)', () => {
  beforeEach(() => {
    api.mockReset()
    api.mockResolvedValue([])
    signInAsMember()
    // The failed-state retry button only shows inside a loaded group.
    useState('groupDetail').value = { group: { name: 'G' }, members: [{}] }
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
      if (isGroup(url)) return Promise.reject(new Error('backend down'))
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

describe('useGroup leaving while a refresh is in flight', () => {
  beforeEach(() => {
    api.mockReset()
    api.mockResolvedValue([])
    vi.stubGlobal('confirm', () => true)
    signInAsMember()
    useState('groupDetail').value = leftGroup
    useMemberLists().value = emptyMemberLists()
  })

  it.each(['leaveGroup', 'deleteGroup'] as const)(
    'does not write the group back when the refresh settles after %s',
    async (action) => {
      const pendingGroup = holdGroupRequest()
      const group = useGroup()

      const poll = group.refreshMemberData()
      await group[action]()
      pendingGroup.resolve(leftGroup)
      await poll

      expect(group.groupDetail.value).toBeNull()
    },
  )

  it('starts fresh requests for a refresh after the leave instead of joining the old one', async () => {
    const pendingGroup = holdGroupRequest()
    const group = useGroup()

    const stalePoll = group.refreshMemberData()
    await group.leaveGroup()
    signInAsMember()
    useState('groupDetail').value = leftGroup
    const groupCallsBefore = groupCalls()
    const librariesBefore = librariesCalls()
    const newRefresh = group.refreshMemberData()

    expect(groupCalls()).toBe(groupCallsBefore + 1)
    expect(librariesCalls()).toBe(librariesBefore + 1)
    pendingGroup.resolve(leftGroup)
    await stalePoll
    await newRefresh
  })

  it('lets a poll join the new refresh even after the old one settles', async () => {
    const oldGroup = deferred<typeof leftGroup>()
    const newGroup = deferred<typeof leftGroup>()
    let groupRequests = 0
    api.mockImplementation((url: string) => {
      if (!isGroup(url)) return Promise.resolve([])
      return ++groupRequests === 1 ? oldGroup.promise : newGroup.promise
    })
    const group = useGroup()

    const stalePoll = group.refreshMemberData()
    await group.leaveGroup()
    signInAsMember()
    useState('groupDetail').value = leftGroup
    const newRefresh = group.refreshMemberData()
    oldGroup.resolve(leftGroup)
    await stalePoll
    const joinedPoll = group.refreshMemberData()

    expect(groupRequests).toBe(2)
    newGroup.resolve(leftGroup)
    await newRefresh
    await joinedPoll
  })
})

describe('useGroup leaving before its navigation finishes', () => {
  beforeEach(() => {
    api.mockReset()
    api.mockResolvedValue([])
    vi.stubGlobal('confirm', () => true)
    signInAsMember()
    useState('groupDetail').value = leftGroup
    useMemberLists().value = emptyMemberLists()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  function holdNavigation() {
    const navigation = deferred<undefined>()
    const push = vi.spyOn(useRouter(), 'push').mockReturnValue(navigation.promise)
    return {
      // Waits until the action has reset the group and is awaiting the navigation.
      reached: () => vi.waitFor(() => expect(push).toHaveBeenCalled()),
      finish: () => navigation.resolve(undefined),
    }
  }

  it.each(['leaveGroup', 'deleteGroup'] as const)(
    'does not write the group back from a refresh started while %s navigates away',
    async (action) => {
      const navigation = holdNavigation()
      const pendingGroup = holdGroupRequest()
      const group = useGroup()

      const leaving = group[action]()
      await navigation.reached()
      const poll = group.refreshMemberData()
      pendingGroup.resolve(leftGroup)
      await poll
      navigation.finish()
      await leaving

      expect(group.groupDetail.value).toBeNull()
    },
  )

  it('does not overwrite the next group opened before that refresh settles', async () => {
    const navigation = holdNavigation()
    const pendingGroup = holdGroupRequest()
    const group = useGroup()
    const nextGroup = { group: { name: 'Next group' }, members: [{}] }

    const leaving = group.leaveGroup()
    await navigation.reached()
    const poll = group.refreshMemberData()
    navigation.finish()
    await leaving
    // The next group's ensureLoaded() fills the header.
    useState('groupDetail').value = nextGroup
    pendingGroup.resolve(leftGroup)
    await poll

    expect(group.groupDetail.value).toEqual(nextGroup)
  })
})

describe('useGroup refreshing as a non-member', () => {
  beforeEach(() => {
    api.mockReset()
    useState('groupMe').value = null
    useState('groupDetail').value = { group: { name: 'Before' }, members: [] }
    useMemberLists().value = emptyMemberLists()
  })

  it('updates the shown group', async () => {
    const synced = { group: { name: 'After' }, members: [] }
    api.mockImplementation((url: string) =>
      isGroup(url) ? Promise.resolve(synced) : Promise.resolve([]),
    )
    const group = useGroup()

    await group.refreshMemberData()

    expect(group.groupDetail.value).toEqual(synced)
  })
})

describe('useGroup leaving while the initial load is in flight', () => {
  beforeEach(() => {
    api.mockReset()
    vi.stubGlobal('confirm', () => true)
    useMemberLists().value = emptyMemberLists()
    useState('groupDetail').value = null
    useState('groupMe').value = null
    useMemberToken().setToken('member-token')
  })

  it.each(['leaveGroup', 'deleteGroup'] as const)(
    'does not write the group or member back when the load settles after %s',
    async (action) => {
      const pendingGroup = deferred<typeof leftGroup>()
      const pendingMe = deferred<MeResponse>()
      api.mockImplementation((url: string, options?: { method?: string }) => {
        if (isGroup(url)) return pendingGroup.promise
        if (url.endsWith('/api/members/me') && !options?.method) return pendingMe.promise
        return Promise.resolve([])
      })
      const group = useGroup()

      const load = group.ensureLoaded()
      // Another load (the layout's or the page's) fills the group first, so the member can leave.
      useState('groupDetail').value = leftGroup
      signInAsMember()
      const member = group.me.value!
      await group[action]()
      pendingGroup.resolve(leftGroup)
      pendingMe.resolve(member)
      await load

      expect(group.groupDetail.value).toBeNull()
      expect(group.me.value).toBeNull()
    },
  )
})

describe('useGroup opening another group in the same tab', () => {
  const groupA = { group: { invite_code: 'ABC', name: 'Group A' }, members: [{}] }
  const groupB = { group: { invite_code: 'XYZ', name: 'Group B' }, members: [{}, {}] }
  const meB = {
    group: { id: 'group-2', invite_code: 'XYZ' },
    member: { id: 'member-2' },
  } as unknown as MeResponse

  // Serves both groups and B's member; `lists` answers the list requests (empty by default).
  function serveGroups(lists: (url: string) => Promise<unknown> = () => Promise.resolve([])) {
    api.mockImplementation((url: string) => {
      if (isGroup(url)) return Promise.resolve(url.endsWith('/XYZ') ? groupB : groupA)
      if (url.endsWith('/api/members/me')) return Promise.resolve(meB)
      return lists(url)
    })
  }

  beforeEach(() => {
    api.mockReset()
    useMemberLists().value = emptyMemberLists()
    // Group A's page was shown, then the member created or joined group B (a new token).
    useState('groupDetail').value = groupA
    signInAsMember()
    useMemberToken().setToken('group-b-token')
    serveGroups()
  })

  afterEach(async () => {
    await useRouter().push('/')
  })

  it("loads the new group's detail and member instead of keeping the previous group's", async () => {
    await useRouter().push('/g/XYZ')
    const group = useGroup()

    await group.ensureLoaded()

    expect(group.groupDetail.value).toEqual(groupB)
    expect(group.me.value).toEqual(meB)
    expect(group.isMember.value).toBe(true)
  })

  it("loads the new group's member lists, not a previous group's request still in flight", async () => {
    const libraryA = [{ member: { id: 'member-1' } }]
    const libraryB = [{ member: { id: 'member-2' } }]
    const pendingLibraryA = deferred<typeof libraryA>()
    serveGroups((url) => {
      if (isLibraries(url)) return pendingLibraryA.promise
      if (url.endsWith('/api/groups/group-2/libraries')) return Promise.resolve(libraryB)
      return Promise.resolve([])
    })
    const group = useGroup()
    // Group A's page polls, then the member opens group B before that poll settles.
    const poll = group.refreshMemberData()
    await useRouter().push('/g/XYZ')

    await group.ensureLoaded()
    pendingLibraryA.resolve(libraryA)
    await poll

    expect(useMemberLists().value.libraries.data).toEqual(libraryB)
  })

  it("keeps the cached group when returning to that group's page", async () => {
    // The invite link's case differs from the stored code's.
    await useRouter().push('/g/abc')
    const group = useGroup()

    await group.ensureLoaded()

    expect(groupCalls()).toBe(0)
    expect(group.groupDetail.value).toEqual(groupA)
  })

  it("shows the member view after joining from the new group's page while the previous member is cached", async () => {
    // A member of group A opened group B's invite link, so B is shown but `me` is still A's.
    useState('groupDetail').value = groupB
    const serve = api.getMockImplementation()!
    api.mockImplementation((url: string) =>
      url.endsWith('/join') ? Promise.resolve({ token: 'group-b-token' }) : serve(url),
    )
    await useRouter().push('/g/XYZ')
    const group = useGroup()

    await group.joinGroup('Player two')

    expect(group.me.value).toEqual(meB)
    expect(group.isMember.value).toBe(true)
  })
})
