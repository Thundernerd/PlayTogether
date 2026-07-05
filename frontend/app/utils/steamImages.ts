import type { GameDetail } from '~/types/api'

/** Use the API-provided header URL when available. */
export function headerImageSrc(url?: string | null): string | null {
  const trimmed = url?.trim()
  return trimmed || null
}

/** Prefix relative API paths (e.g. trailer proxy URLs) with the API base. */
export function resolveApiMediaUrl(path: string, apiBase: string): string {
  if (path.startsWith('http://') || path.startsWith('https://')) {
    return path
  }
  const base = apiBase.replace(/\/$/, '')
  return `${base}${path.startsWith('/') ? path : `/${path}`}`
}

/** Refetch canonical header URL from the game detail endpoint after an image load failure. */
export async function refetchHeaderImage(
  fetchDetail: (appid: number) => Promise<GameDetail>,
  appid: number,
): Promise<string | null> {
  const detail = await fetchDetail(appid)
  return headerImageSrc(detail.header_image_url)
}
