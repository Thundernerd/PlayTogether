export type SyncJobStatus = 'idle' | 'running' | 'complete' | 'error'
export type SyncPhase = 'fetching_wishlist' | 'fetching_games'

export interface SyncStatus {
  status: SyncJobStatus
  phase?: SyncPhase
  progress_done: number
  progress_total: number
  error_message?: string | null
}

export interface Group {
  id: string
  name: string
  invite_code: string
  created_at: string
}

export interface Member {
  id: string
  group_id: string
  display_name: string
  steam_id?: string | null
  created_at: string
}

export interface MemberPublic {
  id: string
  display_name: string
  steam_id?: string | null
  sync_status: SyncStatus
}

export interface GamePlayModes {
  online_coop: boolean
  coop: boolean
  multiplayer: boolean
  remote_play_together: boolean
}

export interface GamePrice {
  initial_cents?: number | null
  final_cents?: number | null
  discount_percent?: number | null
  formatted_initial?: string | null
  formatted_final?: string | null
  discount_end_date?: number | null
}

export interface GameInfo {
  appid: number
  name: string
  short_description: string
  header_image_url: string
  tags: string[]
  is_released?: boolean | null
  price?: GamePrice | null
  play_modes?: GamePlayModes | null
}

export interface GameTrailer {
  name: string
  thumbnail_url?: string | null
  video_url: string
  video_webm_url?: string | null
  category?: number | null
}

export interface GameDetail {
  appid: number
  name: string
  short_description: string
  full_description?: string | null
  header_image_url: string
  tags: string[]
  is_released?: boolean | null
  price?: GamePrice | null
  play_modes?: GamePlayModes | null
  screenshots: string[]
  trailers: GameTrailer[]
  store_url: string
}

export interface WishlistItemView {
  appid: number
  priority: number
  date_added: number
  game?: GameInfo | null
  wants_to_play: boolean
}

export interface MemberWishlist {
  member: MemberPublic
  items: WishlistItemView[]
}

export interface MemberSelectionInfo {
  member_id: string
  display_name: string
}

export interface SharedGame {
  appid: number
  game: GameInfo
  selected_by: MemberSelectionInfo[]
  upvotes: number
  downvotes: number
  net_score: number
  avg_stars?: number | null
  my_vote: number
  my_star_rating?: number | null
}

export interface CreateGroupResponse {
  group: Group
  member: Member
  token: string
}

export interface JoinGroupResponse {
  group: Group
  member: Member
  token: string
}

export interface GroupDetailResponse {
  group: Group
  members: MemberPublic[]
}

export interface MeResponse {
  member: Member
  group: Group
}
