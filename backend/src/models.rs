use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

mod steam_id_serde {
    use super::*;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum SteamIdInput {
        Number(u64),
        String(String),
    }

    pub fn deserialize_optional<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<SteamIdInput>::deserialize(deserializer)?;
        match value {
            None => Ok(None),
            Some(SteamIdInput::Number(id)) => Ok(Some(id)),
            Some(SteamIdInput::String(s)) => s
                .parse::<u64>()
                .map(Some)
                .map_err(serde::de::Error::custom),
        }
    }

    pub fn serialize_optional<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            None => serializer.serialize_none(),
            Some(id) => serializer.serialize_str(&id.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub invite_code: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Member {
    pub id: Uuid,
    pub group_id: Uuid,
    pub display_name: String,
    #[serde(serialize_with = "steam_id_serde::serialize_optional")]
    pub steam_id: Option<u64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemberPublic {
    pub id: Uuid,
    pub display_name: String,
    #[serde(serialize_with = "steam_id_serde::serialize_optional")]
    pub steam_id: Option<u64>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncJobStatus {
    Idle,
    Running,
    Complete,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncPhase {
    FetchingWishlist,
    FetchingGames,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncStatus {
    pub status: SyncJobStatus,
    pub phase: Option<SyncPhase>,
    pub progress_done: i64,
    pub progress_total: i64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GamePrice {
    pub initial_cents: Option<i32>,
    pub final_cents: Option<i32>,
    pub discount_percent: Option<i32>,
    pub formatted_initial: Option<String>,
    pub formatted_final: Option<String>,
    pub discount_end_date: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameInfo {
    pub appid: u32,
    pub name: String,
    pub short_description: String,
    pub header_image_url: String,
    pub tags: Vec<String>,
    pub is_released: Option<bool>,
    pub price: Option<GamePrice>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameTrailer {
    pub name: String,
    pub thumbnail_url: Option<String>,
    pub video_url: String,
    pub video_webm_url: Option<String>,
    pub category: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameDetail {
    pub appid: u32,
    pub name: String,
    pub short_description: String,
    pub full_description: Option<String>,
    pub header_image_url: String,
    pub tags: Vec<String>,
    pub is_released: Option<bool>,
    pub price: Option<GamePrice>,
    pub screenshots: Vec<String>,
    pub trailers: Vec<GameTrailer>,
    pub store_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WishlistItemView {
    pub appid: u32,
    pub priority: i64,
    pub date_added: i64,
    pub game: Option<GameInfo>,
    pub wants_to_play: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemberWishlist {
    pub member: MemberPublic,
    pub items: Vec<WishlistItemView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SharedGame {
    pub appid: u32,
    pub game: GameInfo,
    pub selected_by: Vec<MemberSelectionInfo>,
    pub upvotes: i64,
    pub downvotes: i64,
    pub net_score: i64,
    pub avg_stars: Option<f64>,
    pub my_vote: i64,
    pub my_star_rating: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemberSelectionInfo {
    pub member_id: Uuid,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct JoinGroupRequest {
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRequest {
    #[serde(default, deserialize_with = "steam_id_serde::deserialize_optional")]
    pub steam_id: Option<u64>,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetSelectionRequest {
    pub wants_to_play: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetVoteRequest {
    pub vote: String,
}

#[derive(Debug, Deserialize)]
pub struct SetRatingRequest {
    pub stars: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CreateGroupResponse {
    pub group: Group,
    pub member: Member,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct JoinGroupResponse {
    pub group: Group,
    pub member: Member,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct GroupDetailResponse {
    pub group: Group,
    pub members: Vec<MemberPublic>,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub member: Member,
    pub group: Group,
}
