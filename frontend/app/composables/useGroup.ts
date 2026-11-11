import type {
  GroupDetailResponse,
  MeResponse,
  MemberLibrary,
  MemberWishlist,
  SharedGame,
  SyncStatus,
} from '~/types/api'

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
  const librariesLoading = useState('groupLibrariesLoading', () => false)
  const wishlistsLoading = useState('groupWishlistsLoading', () => false)
  const sharedLoading = useState('groupSharedLoading', () => false)

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

  const anyMemberSyncing = computed(
    () => groupDetail.value?.members.some((m) => m.sync_status.status === 'running') ?? false,
  )

  const groupBasePath = computed(() => `/g/${inviteCode.value}`)

  let librariesInflight: Promise<void> | null = null
  let wishlistsInflight: Promise<void> | null = null
  let sharedInflight: Promise<void> | null = null
  let refreshInflight: Promise<void> | null = null

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

  async function loadWishlists() {
    if (!me.value) return
    if (wishlistsInflight) return wishlistsInflight

    wishlistsLoading.value = true
    wishlistsInflight = (async () => {
      try {
        wishlists.value = await api<MemberWishlist[]>(`/api/groups/${me.value!.group.id}/wishlists`)
      } finally {
        wishlistsLoading.value = false
        wishlistsInflight = null
      }
    })()
    return wishlistsInflight
  }

  async function loadLibraries() {
    if (!me.value) return
    if (librariesInflight) return librariesInflight

    librariesLoading.value = true
    librariesInflight = (async () => {
      try {
        libraries.value = await api<MemberLibrary[]>(`/api/groups/${me.value!.group.id}/libraries`)
      } finally {
        librariesLoading.value = false
        librariesInflight = null
      }
    })()
    return librariesInflight
  }

  async function loadSharedGames() {
    if (!me.value) return
    if (sharedInflight) return sharedInflight

    sharedLoading.value = true
    sharedInflight = (async () => {
      try {
        sharedGames.value = await api<SharedGame[]>(`/api/groups/${me.value!.group.id}/shared-games`)
      } finally {
        sharedLoading.value = false
        sharedInflight = null
      }
    })()
    return sharedInflight
  }

  async function refreshMemberData() {
    if (refreshInflight) return refreshInflight
    refreshInflight = Promise.all([
      loadGroup(),
      loadLibraries(),
      loadWishlists(),
      loadSharedGames(),
    ]).finally(() => {
      refreshInflight = null
    })
    return refreshInflight
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
    anyMemberSyncing,
    loading,
    librariesLoading,
    wishlistsLoading,
    sharedLoading,
    actionLoading,
    error,
    ensureLoaded,
    refreshMemberData,
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
