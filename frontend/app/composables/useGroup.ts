import type {
  GroupDetailResponse,
  MeResponse,
  MemberLibrary,
  MemberWishlist,
  SharedGame,
  SyncStatus,
} from '~/types/api'
import type { Ref } from 'vue'

// In-flight requests, shared by every useGroup() instance like the loading flags, so the
// layout and a page refreshing at once share one request instead of racing on the flags.
// Module scope is safe here: these loads only run on the client (onMounted, polling, actions).
type ListKey = 'libraries' | 'wishlists' | 'shared'
const inflight: Record<ListKey | 'refresh', Promise<void> | null> = {
  libraries: null,
  wishlists: null,
  shared: null,
  refresh: null,
}

export function useGroup() {
  const route = useRoute()
  const router = useRouter()
  const { api } = useApi()
  const { token, setToken, clearToken, syncFromStorage } = useMemberToken()

  const inviteCode = computed(() => String(route.params.inviteCode).toUpperCase())

  const groupDetail = useState<GroupDetailResponse | null>('groupDetail', () => null)
  const me = useState<MeResponse | null>('groupMe', () => null)
  const wishlists = useState<MemberWishlist[]>('groupWishlists', () => [])
  const libraries = useState<MemberLibrary[]>('groupLibraries', () => [])
  const sharedGames = useState<SharedGame[]>('groupSharedGames', () => [])

  // Shared like the data above, so every page's useGroup() sees the loads the layout starts.
  const librariesLoading = useState<boolean>('groupLibrariesLoading', () => false)
  const wishlistsLoading = useState<boolean>('groupWishlistsLoading', () => false)
  const sharedLoading = useState<boolean>('groupSharedLoading', () => false)

  // The last failed request of each list, kept until a load of that list succeeds, so a
  // page can tell "nothing loaded" from "loaded and empty". Not clearing it when a retry or
  // poll starts keeps the failed state steady while a sync keeps polling a failing endpoint.
  const librariesError = useState<string | null>('groupLibrariesError', () => null)
  const wishlistsError = useState<string | null>('groupWishlistsError', () => null)
  const sharedError = useState<string | null>('groupSharedError', () => null)
  const listError = computed(
    () => librariesError.value ?? wishlistsError.value ?? sharedError.value,
  )

  const loading = ref(false)
  const actionLoading = ref(false)
  const error = ref<string | null>(null)

  function resetMemberData() {
    wishlists.value = []
    libraries.value = []
    sharedGames.value = []
    librariesLoading.value = false
    wishlistsLoading.value = false
    sharedLoading.value = false
    librariesError.value = null
    wishlistsError.value = null
    sharedError.value = null
    inflight.libraries = null
    inflight.wishlists = null
    inflight.shared = null
  }

  const isMember = computed(
    () => !!me.value && me.value.group.invite_code.toUpperCase() === inviteCode.value,
  )

  const myWishlist = computed(() =>
    wishlists.value.find((w) => w.member.id === me.value?.member.id),
  )

  const myLibrary = computed(() =>
    libraries.value.find((l) => l.member.id === me.value?.member.id),
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

  async function loadGroup() {
    groupDetail.value = await api<GroupDetailResponse>(`/api/groups/by-invite/${inviteCode.value}`)
  }

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
  function loadList<T>(
    key: ListKey,
    flag: Ref<boolean>,
    errorRef: Ref<string | null>,
    target: Ref<T>,
    path: string,
  ) {
    if (!me.value) return Promise.resolve()
    const pending = inflight[key]
    if (pending) return pending

    flag.value = true
    const request: Promise<void> = api<T>(`/api/groups/${me.value.group.id}/${path}`)
      .then((data) => {
        if (inflight[key] !== request) return
        target.value = data
        errorRef.value = null
      })
      .catch((e: unknown) => {
        if (inflight[key] === request) {
          errorRef.value = e instanceof Error ? e.message : 'Failed to load group data'
        }
      })
      .finally(() => {
        if (inflight[key] !== request) return
        flag.value = false
        inflight[key] = null
      })
    inflight[key] = request
    return request
  }

  const loadWishlists = () =>
    loadList('wishlists', wishlistsLoading, wishlistsError, wishlists, 'wishlists')
  const loadLibraries = () =>
    loadList('libraries', librariesLoading, librariesError, libraries, 'libraries')
  const loadSharedGames = () =>
    loadList('shared', sharedLoading, sharedError, sharedGames, 'shared-games')

  async function refreshMemberData() {
    if (inflight.refresh) return inflight.refresh
    inflight.refresh = Promise.all([
      loadGroup(),
      loadLibraries(),
      loadWishlists(),
      loadSharedGames(),
    ])
      .then(() => {})
      .finally(() => {
        inflight.refresh = null
      })
    return inflight.refresh
  }

  // The failed-state retry button. List failures land in their own error state; a failed
  // group reload only leaves the header stale, so it isn't surfaced as an unhandled rejection.
  function retryListLoads() {
    refreshMemberData().catch(() => {})
  }

  async function ensureLoaded(options: { wishlists?: boolean; libraries?: boolean; shared?: boolean } = {}) {
    syncFromStorage()
    loading.value = true
    error.value = null
    try {
      const [nextGroup, nextMe] = await Promise.all([
        groupDetail.value
          ? undefined
          : api<GroupDetailResponse>(`/api/groups/by-invite/${inviteCode.value}`),
        token.value && !me.value ? fetchMe() : undefined,
      ])
      // Assign and start the list loads in one synchronous step: otherwise Vue renders the
      // member's page before the loading flags are set, flashing its empty / not-synced text.
      if (nextGroup) groupDetail.value = nextGroup
      if (nextMe !== undefined) me.value = nextMe
      if (isMember.value) {
        const fetches: Promise<void>[] = []
        if (options.libraries ?? true) fetches.push(loadLibraries())
        if (options.wishlists ?? true) fetches.push(loadWishlists())
        if (options.shared ?? true) fetches.push(loadSharedGames())
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
    await loadSharedGames()
  }

  async function rate(appid: number, stars: number | null) {
    if (!me.value) return
    await api(`/api/groups/${me.value.group.id}/games/${appid}/rating`, {
      method: 'PUT',
      body: { stars },
    })
    await loadSharedGames()
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
    wishlists,
    libraries,
    sharedGames,
    myWishlist,
    myLibrary,
    isMember,
    syncComplete,
    librarySyncComplete,
    anyMemberSyncing,
    loading,
    librariesLoading,
    wishlistsLoading,
    sharedLoading,
    librariesError,
    wishlistsError,
    sharedError,
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
