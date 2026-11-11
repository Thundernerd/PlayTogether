import type { NitroFetchOptions } from 'nitropack'

export function useApi() {
  const config = useRuntimeConfig()
  const { token } = useMemberToken()

  async function api<T>(path: string, options: NitroFetchOptions<string> = {}): Promise<T> {
    const headers: Record<string, string> = {
      ...(options.headers as Record<string, string> | undefined),
    }

    if (token.value) {
      headers['X-Member-Token'] = token.value
    }

    return $fetch<T>(`${config.public.apiBase}${path}`, {
      ...options,
      headers,
    })
  }

  return { api }
}
