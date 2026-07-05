use std::collections::{HashMap, HashSet};
use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use tracing::info;

use crate::error::AppResult;
use crate::models::GameInfo;
use crate::state::AppState;
use crate::steam::client::{
    AppDetailsData, SteamClient, StoreItem, derive_is_released, derive_is_released_from_app_details,
    header_image_url, normalize_header_image_url,
};

pub struct GameCache;

impl GameCache {
    pub async fn get_cached_appids(
        db: &SqlitePool,
        appids: &[u32],
        ttl_days: i64,
    ) -> AppResult<HashSet<u32>> {
        if appids.is_empty() {
            return Ok(HashSet::new());
        }

        let cutoff = (Utc::now() - Duration::days(ttl_days)).to_rfc3339();
        let placeholders: Vec<String> = appids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT appid FROM steam_games WHERE appid IN ({}) AND fetched_at > ? AND (is_released IS NOT NULL OR raw_json = '{{}}')",
            placeholders.join(", ")
        );

        let mut q = sqlx::query_scalar::<_, i64>(&query);
        for appid in appids {
            q = q.bind(*appid as i64);
        }
        q = q.bind(cutoff);

        let rows = q.fetch_all(db).await?;
        Ok(rows.into_iter().map(|id| id as u32).collect())
    }

    pub async fn load_game(db: &SqlitePool, appid: u32) -> AppResult<Option<GameInfo>> {
        let row = sqlx::query_as::<_, GameRow>(
            "SELECT appid, name, short_description, header_image_url, tags_json, raw_json, is_released FROM steam_games WHERE appid = ?",
        )
        .bind(appid as i64)
        .fetch_optional(db)
        .await?;

        Ok(row.map(|r| r.into_game_info()))
    }

    pub async fn ensure_cached(state: &AppState, appids: &[u32]) -> AppResult<()> {
        Self::fetch_missing(state, appids, |_, _| async {}).await
    }

    pub async fn fetch_missing<F, Fut>(
        state: &AppState,
        appids: &[u32],
        mut on_progress: F,
    ) -> AppResult<()>
    where
        F: FnMut(i64, i64) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        if appids.is_empty() {
            on_progress(0, 0).await;
            return Ok(());
        }

        let cached =
            Self::get_cached_appids(&state.db, appids, state.config.game_cache_ttl_days).await?;
        let needs_fetch: Vec<u32> = appids
            .iter()
            .copied()
            .filter(|id| !cached.contains(id))
            .collect();

        let total = needs_fetch.len() as i64;
        on_progress(0, total).await;

        if needs_fetch.is_empty() {
            return Ok(());
        }

        let client = SteamClient::new(state.http.clone());
        let chunk_size = state.config.steam_chunk_size;
        let delay = state.config.steam_chunk_delay_ms;
        let mut done = 0i64;
        let mut still_missing = Vec::new();

        for chunk in needs_fetch.chunks(chunk_size) {
            let items = client.fetch_store_items(chunk).await?;
            let returned: HashSet<u32> = items.iter().map(|i| i.appid).collect();
            Self::upsert_items(state, &items).await?;

            for appid in chunk {
                if returned.contains(appid) {
                    done += 1;
                    on_progress(done, total).await;
                } else {
                    still_missing.push(*appid);
                }
            }

            if done < total {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            }
        }

        let mut after_individual = Vec::new();
        for appid in still_missing {
            let items = client.fetch_store_items(&[appid]).await?;
            if items.is_empty() {
                after_individual.push(appid);
            } else {
                Self::upsert_items(state, &items).await?;
                done += 1;
                on_progress(done, total).await;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }

        let mut after_details = Vec::new();
        for chunk in after_individual.chunks(10) {
            let details = client.fetch_app_details(chunk).await?;
            for appid in chunk {
                if let Some(detail) = details.get(appid) {
                    Self::upsert_from_app_details(state, *appid, detail).await?;
                    done += 1;
                    on_progress(done, total).await;
                } else {
                    after_details.push(*appid);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }

        for appid in after_details {
            Self::upsert_unavailable(state, appid).await?;
            done += 1;
            on_progress(done, total).await;
        }

        info!(fetched = total, "game cache updated from Steam");
        Ok(())
    }

    async fn upsert_items(state: &AppState, items: &[StoreItem]) -> AppResult<()> {
        let tags = state.tags.read().await;
        let now = Utc::now().to_rfc3339();

        for item in items {
            let appid = item.appid;
            let name = item
                .name
                .clone()
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| format!("Steam app {appid}"));
            let short_description = item
                .basic_info
                .as_ref()
                .and_then(|b| b.short_description.clone())
                .unwrap_or_default();
            let header_image_url = header_image_url(appid, &item.assets);

            let tag_names: Vec<String> = item
                .tags
                .as_ref()
                .map(|tags_list| {
                    tags_list
                        .iter()
                        .filter_map(|t| tags.get(&t.tagid).cloned())
                        .take(10)
                        .collect()
                })
                .unwrap_or_default();

            let tags_json = serde_json::to_string(&tag_names)?;
            let raw_json = serde_json::to_string(item)?;
            let is_released = derive_is_released(item);

            Self::upsert_row(
                state,
                appid,
                &name,
                &short_description,
                &header_image_url,
                &tags_json,
                &raw_json,
                is_released,
                &now,
            )
            .await?;
        }

        Ok(())
    }

    async fn upsert_from_app_details(
        state: &AppState,
        appid: u32,
        detail: &AppDetailsData,
    ) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        let name = if detail.name.is_empty() {
            format!("Steam app {appid}")
        } else {
            detail.name.clone()
        };
        let short_description = detail.short_description.clone().unwrap_or_default();
        let header_image_url = detail
            .header_image
            .clone()
            .unwrap_or_else(|| format!("{STORE_ASSETS_BASE}steam/apps/{appid}/header.jpg"));
        let raw_json = serde_json::to_string(detail)?;
        let is_released = derive_is_released_from_app_details(detail);

        Self::upsert_row(
            state,
            appid,
            &name,
            &short_description,
            &header_image_url,
            "[]",
            &raw_json,
            is_released,
            &now,
        )
        .await
    }

    async fn upsert_unavailable(state: &AppState, appid: u32) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        let name = format!("Steam app {appid}");
        let short_description =
            "This item is no longer available on the Steam store or could not be loaded.";
        let header_image_url = format!("{STORE_ASSETS_BASE}steam/apps/{appid}/header.jpg");

        Self::upsert_row(
            state,
            appid,
            &name,
            short_description,
            &header_image_url,
            "[]",
            "{}",
            None,
            &now,
        )
        .await
    }

    async fn upsert_row(
        state: &AppState,
        appid: u32,
        name: &str,
        short_description: &str,
        header_image_url: &str,
        tags_json: &str,
        raw_json: &str,
        is_released: Option<bool>,
        fetched_at: &str,
    ) -> AppResult<()> {
        let is_released_value = is_released.map(|released| if released { 1 } else { 0 });

        sqlx::query(
            "INSERT INTO steam_games (appid, name, short_description, header_image_url, tags_json, raw_json, is_released, fetched_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(appid) DO UPDATE SET
               name = excluded.name,
               short_description = excluded.short_description,
               header_image_url = excluded.header_image_url,
               tags_json = excluded.tags_json,
               raw_json = excluded.raw_json,
               is_released = excluded.is_released,
               fetched_at = excluded.fetched_at",
        )
        .bind(appid as i64)
        .bind(name)
        .bind(short_description)
        .bind(header_image_url)
        .bind(tags_json)
        .bind(raw_json)
        .bind(is_released_value)
        .bind(fetched_at)
        .execute(&state.db)
        .await?;

        Ok(())
    }
}

const STORE_ASSETS_BASE: &str = "https://shared.fastly.steamstatic.com/store_item_assets/";

struct GameRow {
    appid: i64,
    name: String,
    short_description: String,
    header_image_url: String,
    tags_json: String,
    raw_json: String,
    is_released: Option<i64>,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for GameRow {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            appid: row.try_get("appid")?,
            name: row.try_get("name")?,
            short_description: row.try_get("short_description")?,
            header_image_url: row.try_get("header_image_url")?,
            tags_json: row.try_get("tags_json")?,
            raw_json: row.try_get("raw_json")?,
            is_released: row.try_get("is_released")?,
        })
    }
}

impl GameRow {
    fn into_game_info(self) -> GameInfo {
        let tags: Vec<String> = serde_json::from_str(&self.tags_json).unwrap_or_default();
        let header_image_url = serde_json::from_str::<StoreItem>(&self.raw_json)
            .ok()
            .map(|item| header_image_url(self.appid as u32, &item.assets))
            .or_else(|| {
                serde_json::from_str::<AppDetailsData>(&self.raw_json)
                    .ok()
                    .and_then(|d| d.header_image)
            })
            .unwrap_or_else(|| normalize_header_image_url(&self.header_image_url, self.appid as u32));

        let is_released = serde_json::from_str::<StoreItem>(&self.raw_json)
            .ok()
            .and_then(|item| derive_is_released(&item))
            .or_else(|| {
                serde_json::from_str::<AppDetailsData>(&self.raw_json)
                    .ok()
                    .and_then(|detail| derive_is_released_from_app_details(&detail))
            })
            .or_else(|| self.is_released.map(|value| value != 0));

        GameInfo {
            appid: self.appid as u32,
            name: self.name,
            short_description: self.short_description,
            header_image_url,
            tags,
            is_released,
        }
    }
}

pub async fn load_games_map(
    db: &SqlitePool,
    appids: &[u32],
) -> AppResult<HashMap<u32, GameInfo>> {
    let mut map = HashMap::new();
    for appid in appids {
        if let Some(game) = GameCache::load_game(db, *appid).await? {
            map.insert(*appid, game);
        }
    }
    Ok(map)
}

use sqlx::Row;
