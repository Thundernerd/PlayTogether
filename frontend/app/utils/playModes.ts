import type { GamePlayModes } from '~/types/api'

export type PlayTogetherFilter =
  | 'all'
  | 'online_coop'
  | 'multiplayer_coop'
  | 'unknown'

export function supportsMultiplayerTogether(modes: GamePlayModes): boolean {
  return (
    modes.online_coop ||
    modes.coop ||
    modes.multiplayer ||
    modes.remote_play_together
  )
}

export function matchesPlayTogetherFilter(
  modes: GamePlayModes | null | undefined,
  filter: PlayTogetherFilter,
): boolean {
  switch (filter) {
    case 'all':
      return true
    case 'online_coop':
      return modes?.online_coop === true
    case 'multiplayer_coop':
      return modes != null && supportsMultiplayerTogether(modes)
    case 'unknown':
      return modes == null
  }
}
