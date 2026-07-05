use chrono::{DateTime, Utc};
use rand::Rng;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Group, Member, MemberPublic, SyncJobStatus, SyncPhase, SyncStatus};

pub fn generate_invite_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::rng();
    (0..8)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub async fn member_exists(db: &SqlitePool, member_id: Uuid) -> AppResult<bool> {
    let exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM members WHERE id = ?")
        .bind(member_id.to_string())
        .fetch_optional(db)
        .await?;
    Ok(exists.is_some())
}

pub async fn list_member_ids(db: &SqlitePool, group_id: Uuid) -> AppResult<Vec<Uuid>> {
    let rows: Vec<String> = sqlx::query_scalar("SELECT id FROM members WHERE group_id = ?")
        .bind(group_id.to_string())
        .fetch_all(db)
        .await?;

    rows.into_iter()
        .map(|id| Uuid::parse_str(&id).map_err(|e| AppError::Internal(e.to_string())))
        .collect()
}

pub async fn get_member_by_token(db: &SqlitePool, token: &str) -> AppResult<MemberRow> {
    sqlx::query_as::<_, MemberRow>(
        "SELECT id, group_id, display_name, steam_id, created_at FROM members WHERE token = ?",
    )
    .bind(token)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::Unauthorized)
}

pub async fn get_group(db: &SqlitePool, group_id: Uuid) -> AppResult<GroupRow> {
    sqlx::query_as::<_, GroupRow>(
        "SELECT id, name, invite_code, created_at FROM play_groups WHERE id = ?",
    )
    .bind(group_id.to_string())
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn get_group_by_invite(db: &SqlitePool, invite_code: &str) -> AppResult<GroupRow> {
    sqlx::query_as::<_, GroupRow>(
        "SELECT id, name, invite_code, created_at FROM play_groups WHERE invite_code = ?",
    )
    .bind(invite_code)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn list_group_members(db: &SqlitePool, group_id: Uuid) -> AppResult<Vec<MemberPublic>> {
    let rows = sqlx::query_as::<_, MemberRow>(
        "SELECT id, group_id, display_name, steam_id, created_at FROM members WHERE group_id = ? ORDER BY created_at",
    )
    .bind(group_id.to_string())
    .fetch_all(db)
    .await?;

    let mut members = Vec::new();
    for row in rows {
        let sync = get_sync_status(db, row.id).await?;
        members.push(MemberPublic {
            id: row.id,
            display_name: row.display_name,
            steam_id: row.steam_id.map(|id| id as u64),
            sync_status: sync,
        });
    }
    Ok(members)
}

pub async fn get_sync_status(db: &SqlitePool, member_id: Uuid) -> AppResult<SyncStatus> {
    let row = sqlx::query_as::<_, SyncJobRow>(
        "SELECT status, phase, progress_done, progress_total, error_message FROM member_sync_jobs WHERE member_id = ?",
    )
    .bind(member_id.to_string())
    .fetch_optional(db)
    .await?;

    Ok(match row {
        Some(r) => r.into_sync_status(),
        None => SyncStatus {
            status: SyncJobStatus::Idle,
            phase: None,
            progress_done: 0,
            progress_total: 0,
            error_message: None,
        },
    })
}

pub async fn upsert_sync_job(
    db: &SqlitePool,
    member_id: Uuid,
    status: &str,
    phase: Option<&str>,
    progress_done: i64,
    progress_total: i64,
    error_message: Option<&str>,
) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO member_sync_jobs (member_id, status, phase, progress_done, progress_total, error_message, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(member_id) DO UPDATE SET
           status = excluded.status,
           phase = excluded.phase,
           progress_done = excluded.progress_done,
           progress_total = excluded.progress_total,
           error_message = excluded.error_message,
           updated_at = excluded.updated_at",
    )
    .bind(member_id.to_string())
    .bind(status)
    .bind(phase)
    .bind(progress_done)
    .bind(progress_total)
    .bind(error_message)
    .bind(&now)
    .execute(db)
    .await?;
    Ok(())
}

pub struct GroupRow {
    pub id: String,
    pub name: String,
    pub invite_code: String,
    pub created_at: String,
}

impl GroupRow {
    pub fn into_group(self) -> AppResult<Group> {
        Ok(Group {
            id: Uuid::parse_str(&self.id).map_err(|e| AppError::Internal(e.to_string()))?,
            name: self.name,
            invite_code: self.invite_code,
            created_at: parse_datetime(&self.created_at)?,
        })
    }
}

pub struct MemberRow {
    pub id: Uuid,
    pub group_id: Uuid,
    pub display_name: String,
    pub steam_id: Option<i64>,
    pub created_at: String,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for MemberRow {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: Uuid::parse_str(row.try_get::<String, _>("id")?.as_str())
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            group_id: Uuid::parse_str(row.try_get::<String, _>("group_id")?.as_str())
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            display_name: row.try_get("display_name")?,
            steam_id: row.try_get("steam_id")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

impl MemberRow {
    pub fn into_member(self) -> AppResult<Member> {
        Ok(Member {
            id: self.id,
            group_id: self.group_id,
            display_name: self.display_name,
            steam_id: self.steam_id.map(|id| id as u64),
            created_at: parse_datetime(&self.created_at)?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for GroupRow {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            invite_code: row.try_get("invite_code")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

struct SyncJobRow {
    status: String,
    phase: Option<String>,
    progress_done: i64,
    progress_total: i64,
    error_message: Option<String>,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for SyncJobRow {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            status: row.try_get("status")?,
            phase: row.try_get("phase")?,
            progress_done: row.try_get("progress_done")?,
            progress_total: row.try_get("progress_total")?,
            error_message: row.try_get("error_message")?,
        })
    }
}

impl SyncJobRow {
    fn into_sync_status(self) -> SyncStatus {
        SyncStatus {
            status: match self.status.as_str() {
                "running" => SyncJobStatus::Running,
                "complete" => SyncJobStatus::Complete,
                "error" => SyncJobStatus::Error,
                _ => SyncJobStatus::Idle,
            },
            phase: self.phase.as_deref().map(|p| match p {
                "fetching_wishlist" => SyncPhase::FetchingWishlist,
                "fetching_games" => SyncPhase::FetchingGames,
                _ => SyncPhase::FetchingWishlist,
            }),
            progress_done: self.progress_done,
            progress_total: self.progress_total,
            error_message: self.error_message,
        }
    }
}

fn parse_datetime(s: &str) -> AppResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| AppError::Internal(e.to_string()))
}

use sqlx::Row;
