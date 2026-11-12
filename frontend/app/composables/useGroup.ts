import type { GroupDetailResponse, MeResponse, SyncStatus } from '~/types/api'
import type { ListData, ListKey, MemberListState } from '~/composables/useMemberList'

// In-flight requests, shared by every useGroup() instance like the loading flags, so the
// layout and a page refreshing at once share one request instead of racing on the flags.
// Module scope is safe here: these loads only run on the client (onMounted, polling, actions).
const inflight: Record<ListKey | 'refresh', Promise<void> | null> = {
  libraries: null,
  wishlists: null,
  shared: null,
  refresh: null,
}

// Bumped by resetMemberData() (the member left), so an ensureLoaded() that started before
// the reset knows its group and member responses are stale. Shared like the requests.
let resets = 0

const listPaths: Record<ListKey, string> = {
  libraries: 'libraries',
  wishlists: 'wishlists',
  shared: 'shared-games',
}

export function useGroup() {
  const route = useRoute()
  const router = useRouter()
  const { api } = useApi()
  const { token, setToken, clearToken, syncFromStorage } = useMemberToken()

  const inviteCode = computed(() => String(route.params.inviteCode).toUpperCase())

  const groupDetail = useState<GroupDetailResponse | null>('groupDetail', () => null)
  const me = useState<MeResponse | null>('groupMe', () => null)
  // Each list's data, loading flag and error, read through useMemberList(key).
  const lists = useMemberLists()
  // The first error of a list that never loaded. A failed refresh of a loaded list is left
  // to the "couldn't refresh" hints, which keep its last count and items on show.
  const listError = computed(
    () =>
      LIST_KEYS.map((key) => lists.value[key])
        .find((list) => list.error !== null && !list.loaded)?.error ?? null,
  )

  const loading = ref(false)
  const actionLoading = ref(false)
  const error = ref<string | null>(null)

  function resetMemberData() {
    resets++
    lists.value = emptyMemberLists()
    inflight.libraries = null
    inflight.wishlists = null
    inflight.shared = null
    inflight.refresh = null
  }

  const isMember = computed(
    () => !!me.value && me.value.group.invite_code.toUpperCase() === inviteCode.value,
  )

  const myWishlist = computed(() =>
    lists.value.wishlists.data.find((w) => w.member.id === me.value?.member.id),
  )

  const myLibrary = computed(() =>
    lists.value.libraries.data.find((l) => l.member.id === me.value?.member.id),
  )

  const syncComplete = computed(
    () => !!me.value && myWishlist.value?.member.sync_status.status === 'complete',
  )

  // Same sync job as syncComplete, read from /libraries so the library page needn't wait on /wishlists.
  const librarySyncComplete = computed(
    () => myLibrary.value?.member.sync_status.status === 'complete',
  )

  const anyMemberSyncing = computed(
    () => groupDetail.value?.members.some((m) => m.sync_status.status === 'running') ?? false,
  )

  const groupBasePath = computed(() => `/g/${inviteCode.value}`)

  async function fetchMe(): Promise<MeResponse | null> {
    try {
      return await api<MeResponse>('/api/members/me')
    } catch {
      return null
    }
  }

  // One request per list at a time, shared across instances. A request that resetMemberData()
  // has superseded no longer writes its data or clears the flag of the one that replaced it.
  // A failure lands in the list's error state instead of rejecting, so callers (polling,
  // refreshes) carry on and each page shows its own failed state.
  // The list's entry is looked up again after each await rather than held across it, since
  // resetMemberData() replaces the whole record.
  function loadList<K extends ListKey>(key: K) {
    if (!me.value) return Promise.resolve()
    const pending = inflight[key]
    if (pending) return pending

    const list = (): MemberListState<K> => lists.value[key]
    list().loading = true
    const request: Promise<void> = api<ListData[K]>(
      `/api/groups/${me.value.group.id}/${listPaths[key]}`,
    )
      .then((data) => {
        if (inflight[key] !== request) return
        list().data = data
        list().loaded = true
        list().error = null
      })
      .catch((e: unknown) => {
        if (inflight[key] === request) {
          list().error = e instanceof Error ? e.message : 'Failed to load group data'
        }
      })
      .finally(() => {
        if (inflight[key] !== request) return
        list().loading = false
        inflight[key] = null
      })
    inflight[key] = request
    return request
  }

  // One refresh at a time, shared like the list requests. Once resetMemberData() has
  // superseded it (the member left), it no longer writes the left group back or clears the
  // refresh that replaced it. It reloads the group only if one is shown when it starts:
  // loading one from nothing is ensureLoaded()'s job, and no group means the member left,
  // possibly while the route still holds the old invite code because the navigation away
  // hasn't finished. Fetching that code could write the left group over the next one opened.
  async function refreshMemberData() {
    if (inflight.refresh) return inflight.refresh
    const reloadGroup = groupDetail.value
      ? api<GroupDetailResponse>(`/api/groups/by-invite/${inviteCode.value}`).then((detail) => {
          if (inflight.refresh === refresh) groupDetail.value = detail
        })
      : undefined
    const refresh: Promise<void> = Promise.all([
      reloadGroup,
      loadList('libraries'),
      loadList('wishlists'),
      loadList('shared'),
    ])
      .then(() => {})
      .finally(() => {
        if (inflight.refresh === refresh) inflight.refresh = null
      })
    inflight.refresh = refresh
    return refresh
  }

  // The failed-state retry button. List failures land in their own error state; a failed
  // group reload only leaves the header stale, so it isn't surfaced as an unhandled rejection.
  // A click while a poll's request is in flight joins it, but that request started before the
  // click, so once it settles each joined list that still failed is loaded again; otherwise
  // the click would be lost. A list whose request the click started is loaded only once.
  // The joined list requests are awaited on their own because the refresh rejects as soon as
  // the group reload fails, possibly before they settle (they never reject).
  async function retryListLoads() {
    const joined = LIST_KEYS.filter((key) => inflight.refresh || inflight[key])
    const joinedRequests = joined.map((key) => inflight[key])
    await refreshMemberData().catch(() => {})
    await Promise.all(joinedRequests)
    await Promise.all(
      joined.filter((key) => lists.value[key].error !== null).map((key) => loadList(key)),
    )
  }

  async function ensureLoaded(options: { wishlists?: boolean; libraries?: boolean; shared?: boolean } = {}) {
    syncFromStorage()
    loading.value = true
    error.value = null
    const resetsAtStart = resets
    try {
      const [nextGroup, nextMe] = await Promise.all([
        groupDetail.value
          ? undefined
          : api<GroupDetailResponse>(`/api/groups/by-invite/${inviteCode.value}`),
        token.value && !me.value ? fetchMe() : undefined,
      ])
      // The member left while these were in flight: don't write the left group or member back.
      if (resets !== resetsAtStart) return
      // Assign and start the list loads in one synchronous step: otherwise Vue renders the
      // member's page before the loading flags are set, flashing its empty / not-synced text.
      if (nextGroup) groupDetail.value = nextGroup
      if (nextMe !== undefined) me.value = nextMe
      if (isMember.value) {
        const fetches: Promise<void>[] = []
        if (options.libraries ?? true) fetches.push(loadList('libraries'))
        if (options.wishlists ?? true) fetches.push(loadList('wishlists'))
        if (options.shared ?? true) fetches.push(loadList('shared'))
        await Promise.all(fetches)
      }
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : 'Failed to load group'
    } finally {
      loading.value = false
    }
  }

  async function joinGroup(displayName: string) {
    actionLoading.value = true
    error.value = null
    try {
      const res = await api<{ token: string }>(`/api/groups/by-invite/${inviteCode.value}/join`, {
        method: 'POST',
        body: { display_name: displayName },
      })
      setToken(res.token)
      await ensureLoaded()
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : 'Failed to join group'
    } finally {
      actionLoading.value = false
    }
  }

  async function leaveGroup() {
    if (!me.value || !groupDetail.value) return
    const others = groupDetail.value.members.length - 1
    const message =
      others > 0
        ? `Leave "${groupDetail.value.group.name}"? Your play-together picks and votes will be removed.`
        : `Leave "${groupDetail.value.group.name}"? You are the last member, so the group will be deleted.`
    if (!confirm(message)) {
      return
    }

    actionLoading.value = true
    error.value = null
    try {
      await api('/api/members/me', { method: 'DELETE' })
      clearToken()
      groupDetail.value = null
      me.value = null
      resetMemberData()
      await router.push('/')
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : 'Failed to leave group'
    } finally {
      actionLoading.value = false
    }
  }

  async function deleteGroup() {
    if (!me.value || !groupDetail.value) return
    if (!confirm(`Delete "${groupDetail.value.group.name}"? This cannot be undone.`)) {
      return
    }

    actionLoading.value = true
    error.value = null
    try {
      await api(`/api/groups/${me.value.group.id}`, { method: 'DELETE' })
      clearToken()
      groupDetail.value = null
      me.value = null
      resetMemberData()
      await router.push('/')
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : 'Failed to delete group'
    } finally {
      actionLoading.value = false
    }
  }

  async function saveSteamId(steamId: string) {
    if (!me.value) return
    actionLoading.value = true
    error.value = null
    try {
      if (!/^\d+$/.test(steamId.trim())) {
        throw new Error('Steam ID must be a numeric ID')
      }
      me.value = await api<MeResponse>('/api/members/me', {
        method: 'PATCH',
        body: { steam_id: steamId.trim() },
      })
      await refreshMemberData()
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : 'Failed to save Steam ID'
    } finally {
      actionLoading.value = false
    }
  }

  async function refreshWishlistSync() {
    actionLoading.value = true
    error.value = null
    try {
      await api('/api/members/me/refresh-wishlist', { method: 'POST' })
      await refreshMemberData()
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : 'Failed to refresh wishlist'
    } finally {
      actionLoading.value = false
    }
  }

  async function toggleSelection(appid: number, wantsToPlay: boolean) {
    await api(`/api/members/me/selections/${appid}`, {
      method: 'PUT',
      body: { wants_to_play: wantsToPlay },
    })
    await refreshMemberData()
  }

  async function vote(appid: number, vote: 'up' | 'down' | 'none') {
    if (!me.value) return
    await api(`/api/groups/${me.value.group.id}/games/${appid}/vote`, {
      method: 'PUT',
      body: { vote },
    })
    await loadList('shared')
  }

  async function rate(appid: number, stars: number | null) {
    if (!me.value) return
    await api(`/api/groups/${me.value.group.id}/games/${appid}/rating`, {
      method: 'PUT',
      body: { stars },
    })
    await loadList('shared')
  }

  async function onSyncUpdate(_status: SyncStatus) {
    await refreshMemberData()
  }

  const inviteLink = computed(() =>
    import.meta.client ? `${window.location.origin}/g/${inviteCode.value}` : '',
  )

  async function copyInvite() {
    if (!inviteLink.value) return
    await navigator.clipboard.writeText(inviteLink.value)
  }

  return {
    inviteCode,
    groupBasePath,
    groupDetail,
    me,
    myWishlist,
    myLibrary,
    isMember,
    syncComplete,
    librarySyncComplete,
    anyMemberSyncing,
    loading,
    listError,
    actionLoading,
    error,
    ensureLoaded,
    refreshMemberData,
    retryListLoads,
    joinGroup,
    leaveGroup,
    deleteGroup,
    saveSteamId,
    refreshWishlistSync,
    toggleSelection,
    vote,
    rate,
    onSyncUpdate,
    copyInvite,
  }
}
