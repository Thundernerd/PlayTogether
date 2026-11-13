// Keeps its pre-Squadlist name on purpose so existing members keep their tokens.
const TOKEN_KEY = 'playtogether_member_token'

export function useMemberToken() {
  const token = useState<string | null>('memberToken', () => null)

  function syncFromStorage() {
    if (!import.meta.client) return
    const stored = localStorage.getItem(TOKEN_KEY)
    if (stored) {
      token.value = stored
    }
  }

  // useState is seeded as null during SSR; restore from localStorage on the client.
  if (import.meta.client) {
    syncFromStorage()
  }

  onMounted(syncFromStorage)

  function setToken(value: string) {
    token.value = value
    if (import.meta.client) {
      localStorage.setItem(TOKEN_KEY, value)
    }
  }

  function clearToken() {
    token.value = null
    if (import.meta.client) {
      localStorage.removeItem(TOKEN_KEY)
    }
  }

  return { token, setToken, clearToken, syncFromStorage }
}
