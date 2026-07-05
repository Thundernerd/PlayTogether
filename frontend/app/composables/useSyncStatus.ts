import type { SyncStatus } from '~/types/api'

export function useSyncStatus(onUpdate?: (status: SyncStatus) => void) {
  const { api } = useApi()
  const polling = ref(false)
  let timer: ReturnType<typeof setInterval> | null = null

  async function pollOnce() {
    const status = await api<SyncStatus>('/api/members/me/sync-status')
    onUpdate?.(status)
    return status
  }

  function startPolling() {
    if (polling.value) return
    polling.value = true

    const tick = async () => {
      try {
        const status = await pollOnce()
        if (status.status === 'complete' || status.status === 'error') {
          stopPolling()
        }
      } catch {
        stopPolling()
      }
    }

    tick()
    timer = setInterval(tick, 2000)
  }

  function stopPolling() {
    polling.value = false
    if (timer) {
      clearInterval(timer)
      timer = null
    }
  }

  onUnmounted(stopPolling)

  return { polling, pollOnce, startPolling, stopPolling }
}
