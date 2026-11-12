use chrono::Utc;
use std::collections::HashMap;
use tracing::info;

use crate::error::AppResult;
use crate::state::AppState;
use crate::steam::client::SteamClient;

pub struct TagCache;

impl TagCache {
    pub async fn warm(state: &AppState) -> AppResult<()> {
        let stored = sqlx::query_as::<_, TagCacheRow>(
            "SELECT version_hash, tags_json FROM steam_tag_cache WHERE id = 1",
        )
        .fetch_optional(&state.db)
        .await?;

        if let Some(row) = stored {
            if let Ok(map) = Self::parse_tags_json(&row.tags_json) {
                *state.tags.write().await = map;
                info!(version = %row.version_hash, "loaded tag cache from database");
            }
        }

        Self::refresh_if_needed(state).await
    }

    pub async fn refresh_if_needed(state: &AppState) -> AppResult<()> {
        let client = SteamClient::new(state.http.clone());
        let remote = client.fetch_tag_list().await?;

        let stored_hash: Option<String> =
            sqlx::query_scalar("SELECT version_hash FROM steam_tag_cache WHERE id = 1")
                .fetch_optional(&state.db)
                .await?;

        if stored_hash.as_deref() == Some(remote.response.version_hash.as_str()) {
            return Ok(());
        }

        let tags_json = serde_json::to_string(&remote.response.tags)?;
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO steam_tag_cache (id, version_hash, tags_json, fetched_at)
             VALUES (1, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               version_hash = excluded.version_hash,
               tags_json = excluded.tags_json,
               fetched_at = excluded.fetched_at",
        )
        .bind(&remote.response.version_hash)
        .bind(&tags_json)
        .bind(&now)
        .execute(&state.db)
        .await?;

        *state.tags.write().await = Self::parse_tags_json(&tags_json)?;
        info!(version = %remote.response.version_hash, "refreshed tag cache from Steam");

        Ok(())
    }

    fn parse_tags_json(json: &str) -> AppResult<HashMap<u32, String>> {
        let entries: Vec<TagEntryJson> = serde_json::from_str(json)?;
        Ok(entries.into_iter().map(|t| (t.tagid, t.name)).collect())
    }
}

struct TagCacheRow {
    version_hash: String,
    tags_json: String,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for TagCacheRow {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            version_hash: row.try_get("version_hash")?,
            tags_json: row.try_get("tags_json")?,
        })
    }
}

#[derive(serde::Deserialize)]
struct TagEntryJson {
    tagid: u32,
    name: String,
}

use sqlx::Row;
