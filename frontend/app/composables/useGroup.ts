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

  const loading = ref(false)
  const actionLoading = ref(false)
  const error = ref<string | null>(null)

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

  async function loadGroup() {
    groupDetail.value = await api<GroupDetailResponse>(`/api/groups/by-invite/${inviteCode.value}`)
  }

  async function loadMe() {
    if (!token.value) {
      me.value = null
      return
    }
    try {
      me.value = await api<MeResponse>('/api/members/me')
    } catch {
      me.value = null
    }
  }

  async function loadWishlists() {
    if (!me.value) return
    wishlists.value = await api<MemberWishlist[]>(`/api/groups/${me.value.group.id}/wishlists`)
  }

  async function loadLibraries() {
    if (!me.value) return
    libraries.value = await api<MemberLibrary[]>(`/api/groups/${me.value.group.id}/libraries`)
  }

  async function loadSharedGames() {
    if (!me.value) return
    sharedGames.value = await api<SharedGame[]>(`/api/groups/${me.value.group.id}/shared-games`)
  }

  async function refreshMemberData() {
    await Promise.all([loadGroup(), loadLibraries(), loadWishlists(), loadSharedGames()])
  }

  async function ensureLoaded(options: { wishlists?: boolean; libraries?: boolean; shared?: boolean } = {}) {
    syncFromStorage()
    loading.value = true
    error.value = null
    try {
      if (!groupDetail.value) {
        await loadGroup()
      }
      if (token.value && !me.value) {
        await loadMe()
      }
      if (isMember.value) {
        if (options.libraries ?? true) {
          await loadLibraries()
        }
        if (options.wishlists ?? true) {
          await loadWishlists()
        }
        if (options.shared ?? true) {
          await loadSharedGames()
        }
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
      wishlists.value = []
      libraries.value = []
      sharedGames.value = []
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
    actionLoading,
    error,
    ensureLoaded,
    refreshMemberData,
    joinGroup,
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
