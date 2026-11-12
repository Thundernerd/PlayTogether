use chrono::{DateTime, Duration, Utc};
use futures_util::future::join_all;
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};

use crate::error::{AppError, AppResult};
use crate::models::{GameDetail, GameInfo, GamePlayModes, GamePrice, GameTrailer};
use crate::state::AppState;
use crate::steam::categories::play_modes_from_item;
use crate::steam::client::{
    derive_is_released, derive_is_released_from_app_details, store_item_has_details,
    AppDetailsData, SteamClient, StoreItem,
};
use crate::steam::game_refresh::{FetchClaim, InFlightFetch, OwnedFetch};
use crate::steam::media_urls::{
    collect_trailers, header_image_url, header_image_url_from_app_details,
    header_image_url_from_item, is_allowed_video_cdn_url, normalize_header_image_url,
    screenshot_urls_from_item, trailer_entries, trailer_proxy_video_url, trailer_video_mp4_url,
    trailer_video_webm_url, STORE_ASSETS_BASE,
};
use crate::steam::pricing::price_from_store_item;
use crate::steam::store::SteamStore;

pub struct GameCache;

/// Which `steam_games` rows to match by `fetched_at`.
#[derive(Clone, Copy)]
enum FetchedAt {
    Any,
    After(DateTime<Utc>),
    AtOrBefore(DateTime<Utc>),
}

impl GameCache {
    pub async fn get_cached_appids(
        db: &SqlitePool,
        appids: &[u32],
        ttl_days: i64,
    ) -> AppResult<HashSet<u32>> {
        let cutoff = Utc::now() - Duration::days(ttl_days);
        Self::appids_by_fetched_at(db, appids, FetchedAt::After(cutoff)).await
    }

    pub async fn get_price_stale_appids(
        db: &SqlitePool,
        appids: &[u32],
        ttl_hours: i64,
    ) -> AppResult<HashSet<u32>> {
        let cutoff = Utc::now() - Duration::hours(ttl_hours);
        Self::appids_by_fetched_at(db, appids, FetchedAt::AtOrBefore(cutoff)).await
    }

    /// The `appids` with a `steam_games` row, however old.
    async fn get_stored_appids(db: &SqlitePool, appids: &[u32]) -> AppResult<HashSet<u32>> {
        Self::appids_by_fetched_at(db, appids, FetchedAt::Any).await
    }

    /// The `appids` that are due for a fetch from Steam: no row, or a row past
    /// the game or the price TTL.
    async fn get_due_appids(state: &AppState, appids: &[u32]) -> AppResult<HashSet<u32>> {
        let cached =
            Self::get_cached_appids(&state.db, appids, state.config.game_cache_ttl_days).await?;
        let price_stale =
            Self::get_price_stale_appids(&state.db, appids, state.config.price_cache_ttl_hours)
                .await?;
        Ok(appids
            .iter()
            .copied()
            .filter(|id| !cached.contains(id) || price_stale.contains(id))
            .collect())
    }

    /// The `appids` with a `steam_games` row whose `fetched_at` matches.
    async fn appids_by_fetched_at(
        db: &SqlitePool,
        appids: &[u32],
        fetched_at: FetchedAt,
    ) -> AppResult<HashSet<u32>> {
        if appids.is_empty() {
            return Ok(HashSet::new());
        }

        let placeholders: Vec<String> = appids.iter().map(|_| "?".to_string()).collect();
        let mut query = format!(
            "SELECT appid FROM steam_games WHERE appid IN ({})",
            placeholders.join(", ")
        );
        let cutoff = match fetched_at {
            FetchedAt::Any => None,
            FetchedAt::After(cutoff) => {
                query.push_str(" AND fetched_at > ?");
                Some(cutoff)
            }
            FetchedAt::AtOrBefore(cutoff) => {
                query.push_str(" AND fetched_at <= ?");
                Some(cutoff)
            }
        };

        let mut q = sqlx::query_scalar::<_, i64>(&query);
        for appid in appids {
            q = q.bind(*appid as i64);
        }
        if let Some(cutoff) = cutoff {
            q = q.bind(cutoff.to_rfc3339());
        }

        let rows = q.fetch_all(db).await?;
        Ok(rows.into_iter().map(|id| id as u32).collect())
    }

    pub async fn load_game(db: &SqlitePool, appid: u32) -> AppResult<Option<GameInfo>> {
        let row = Self::load_row(db, appid).await?;
        Ok(row.map(|r| r.into_game_info()))
    }

    pub async fn ensure_detail_cached(state: &AppState, appid: u32) -> AppResult<()> {
        let client = SteamClient::new(state.http.clone());
        Self::ensure_detail_cached_from(state, &client, appid).await
    }

    async fn ensure_detail_cached_from(
        state: &AppState,
        client: &impl SteamStore,
        appid: u32,
    ) -> AppResult<()> {
        let row = Self::load_row(&state.db, appid).await?;
        if let Some(row) = &row {
            if let Ok(item) = serde_json::from_str::<StoreItem>(&row.raw_json) {
                if store_item_has_details(&item) {
                    return Ok(());
                }
            }
        }

        let items = client.fetch_store_item_details(appid).await?;
        if items.is_empty() {
            // Steam no longer lists the app; show what the cache already has.
            return if row.is_some() {
                Ok(())
            } else {
                Err(AppError::NotFound)
            };
        }

        Self::upsert_items(state, &items).await?;
        Ok(())
    }

    pub async fn load_game_detail(state: &AppState, appid: u32) -> AppResult<Option<GameDetail>> {
        let Some(row) = Self::load_row(&state.db, appid).await? else {
            return Ok(None);
        };

        let tags = state.tags.read().await;
        Ok(Some(row.into_game_detail(appid, &tags)))
    }

    pub async fn trailer_cdn_url(
        db: &SqlitePool,
        appid: u32,
        index: usize,
        webm: bool,
    ) -> AppResult<String> {
        let row = Self::load_row(db, appid).await?.ok_or(AppError::NotFound)?;
        let item =
            serde_json::from_str::<StoreItem>(&row.raw_json).map_err(|_| AppError::NotFound)?;
        let trailers = item.trailers.as_ref().ok_or(AppError::NotFound)?;
        let trailer = collect_trailers(trailers)
            .get(index)
            .copied()
            .ok_or(AppError::NotFound)?;

        let url = if webm {
            trailer_video_webm_url(trailer).ok_or(AppError::NotFound)?
        } else {
            trailer_video_mp4_url(trailer).ok_or(AppError::NotFound)?
        };

        if !is_allowed_video_cdn_url(&url) {
            return Err(AppError::Internal("invalid trailer CDN URL".into()));
        }

        Ok(url)
    }

    pub async fn backfill_header_urls(db: &SqlitePool) -> AppResult<()> {
        let rows = sqlx::query_as::<_, (i64, String, String)>(
            "SELECT appid, header_image_url, raw_json FROM steam_games WHERE raw_json != '{}'",
        )
        .fetch_all(db)
        .await?;

        let mut updated = 0usize;
        for (appid, stored_header, raw_json) in rows {
            let appid = appid as u32;
            let resolved = serde_json::from_str::<StoreItem>(&raw_json)
                .ok()
                .map(|item| header_image_url_from_item(&item))
                .or_else(|| {
                    serde_json::from_str::<AppDetailsData>(&raw_json)
                        .ok()
                        .and_then(|detail| header_image_url_from_app_details(&detail))
                });

            if let Some(resolved) = resolved {
                if resolved != stored_header {
                    sqlx::query("UPDATE steam_games SET header_image_url = ? WHERE appid = ?")
                        .bind(&resolved)
                        .bind(appid as i64)
                        .execute(db)
                        .await?;
                    updated += 1;
                }
            }
        }

        if updated > 0 {
            info!(updated, "backfilled header_image_url from raw_json");
        }

        Ok(())
    }

    async fn load_row(db: &SqlitePool, appid: u32) -> AppResult<Option<GameRow>> {
        let row = sqlx::query_as::<_, GameRow>(
            "SELECT appid, name, short_description, header_image_url, tags_json, raw_json, is_released FROM steam_games WHERE appid = ?",
        )
        .bind(appid as i64)
        .fetch_optional(db)
        .await?;

        Ok(row)
    }

    pub async fn ensure_cached(state: &AppState, appids: &[u32]) -> AppResult<()> {
        let client = SteamClient::new(state.http.clone());
        if Self::ensure_cached_from(state, &client, appids).await? {
            let state = state.clone();
            tokio::spawn(async move { Self::refresh_queued(&state, &client).await });
        }
        Ok(())
    }

    /// The background refresh: refetches queued apps from Steam until the
    /// queue is empty. `fetch_missing` skips apps refreshed in the meantime.
    /// Apps Steam fails for are held back from the queue for a cooldown.
    async fn refresh_queued(state: &AppState, client: &impl SteamStore) {
        let mut worker = state.game_refresh.start_worker();
        while let Some(batch) = worker.next_batch() {
            info!(
                count = batch.len(),
                "refreshing stale game cache rows in the background"
            );
            match Self::fetch_missing(state, client, &batch, |_, _| async {}).await {
                Ok(failed) => state.fetch_cooldown.record_failed(&failed),
                Err(error) => {
                    warn!(%error, "background game cache refresh failed; rows stay due for refresh");
                    // Which apps failed is unknown, so hold back the whole batch.
                    state.fetch_cooldown.record_failed(&batch);
                }
            }
        }
    }

    /// Fetches the `appids` with no cached row and waits for them. Apps with
    /// no row that Steam failed for are left out until their cooldown has
    /// passed, so list loads don't refetch them every time. Apps whose
    /// row is past a TTL are served as cached and queued for the background
    /// refresh. Returns true when the caller must start the refresh worker.
    #[must_use = "when true, the caller must start the refresh worker"]
    async fn ensure_cached_from(
        state: &AppState,
        client: &impl SteamStore,
        appids: &[u32],
    ) -> AppResult<bool> {
        let stored = Self::get_stored_appids(&state.db, appids).await?;
        let missing: Vec<u32> = appids
            .iter()
            .copied()
            .filter(|id| !stored.contains(id))
            .collect();
        Self::fetch_uncached(state, client, missing).await?;

        let due = Self::get_due_appids(state, appids).await?;
        let stale: Vec<u32> = appids
            .iter()
            .copied()
            .filter(|id| stored.contains(id) && due.contains(id))
            .collect();

        Ok(state.game_refresh.enqueue(&stale, &state.fetch_cooldown))
    }

    /// Fetches the `appids`, which have no row, from Steam, except the ones in
    /// their cooldown. An app another list load is fetching already is not
    /// fetched again: this waits for that fetch instead, and fetches the app
    /// itself only when that fetch was dropped before it stored the app or
    /// started its cooldown.
    async fn fetch_uncached(
        state: &AppState,
        client: &impl SteamStore,
        mut missing: Vec<u32>,
    ) -> AppResult<()> {
        loop {
            missing = state.fetch_cooldown.not_cooling_down(&missing);
            if missing.is_empty() {
                return Ok(());
            }
            let FetchClaim {
                fetch,
                in_flight_elsewhere,
            } = state.game_refresh.claim_fetches(&missing);
            if in_flight_elsewhere.is_empty() {
                return Self::fetch_owned(state, client, fetch).await;
            }
            let (own, _) = tokio::join!(
                Self::fetch_owned(state, client, fetch),
                join_all(in_flight_elsewhere.into_iter().map(InFlightFetch::ended)),
            );
            own?;
            let stored = Self::get_stored_appids(&state.db, &missing).await?;
            missing.retain(|id| !stored.contains(id));
        }
    }

    /// Fetches the apps `fetch` claimed and releases the claim on each one as
    /// soon as it is settled, so a list load waiting for only some of them
    /// does not wait for the rest. The cooldown for an app Steam failed for
    /// starts before its claim is released.
    async fn fetch_owned(
        state: &AppState,
        client: &impl SteamStore,
        mut fetch: OwnedFetch<'_>,
    ) -> AppResult<()> {
        let appids = fetch.appids().to_vec();
        Self::fetch_missing_settling(
            state,
            client,
            &appids,
            |_, _| async {},
            |settled| {
                if let Settled::Failed(failed) = settled {
                    state.fetch_cooldown.record_failed(failed);
                }
                fetch.release(settled.appids());
            },
        )
        .await?;
        Ok(())
    }

    /// Fetches the `appids` that are due from Steam and caches them. Apps
    /// Steam fails for keep the row they had; returns their appids. When a
    /// chunk request fails, its apps are asked for one at a time, unless Steam
    /// rate limited it: then every app in the chunk counts as failed. A rate
    /// limit while asking one at a time fails the apps not asked for yet.
    pub async fn fetch_missing<F, Fut>(
        state: &AppState,
        client: &impl SteamStore,
        appids: &[u32],
        on_progress: F,
    ) -> AppResult<Vec<u32>>
    where
        F: FnMut(i64, i64) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        Self::fetch_missing_settling(state, client, appids, on_progress, |_| {}).await
    }

    /// `fetch_missing`, telling `on_settled` about each app as soon as the
    /// walk is done with it, rather than once the whole walk has ended.
    async fn fetch_missing_settling<F, Fut>(
        state: &AppState,
        client: &impl SteamStore,
        appids: &[u32],
        mut on_progress: F,
        mut on_settled: impl FnMut(Settled<'_>),
    ) -> AppResult<Vec<u32>>
    where
        F: FnMut(i64, i64) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        if appids.is_empty() {
            on_progress(0, 0).await;
            return Ok(Vec::new());
        }

        let due = Self::get_due_appids(state, appids).await?;
        let needs_fetch: Vec<u32> = appids
            .iter()
            .copied()
            .filter(|id| due.contains(id))
            .collect();
        let not_due: Vec<u32> = appids
            .iter()
            .copied()
            .filter(|id| !due.contains(id))
            .collect();
        if !not_due.is_empty() {
            on_settled(Settled::Cached(&not_due));
        }

        let total = needs_fetch.len() as i64;
        on_progress(0, total).await;

        if needs_fetch.is_empty() {
            return Ok(Vec::new());
        }

        let chunk_size = state.config.steam_chunk_size;
        let delay = tokio::time::Duration::from_millis(state.config.steam_chunk_delay_ms);
        let mut progress = WalkProgress::new(total, on_progress, on_settled);
        let mut still_missing = Vec::new();

        for chunk in needs_fetch.chunks(chunk_size) {
            match client.fetch_store_items(chunk).await {
                Ok(items) => {
                    let items = Self::requested_items(chunk, items);
                    let returned: HashSet<u32> = items.iter().map(|i| i.appid).collect();
                    Self::upsert_items(state, &items).await?;

                    for appid in chunk {
                        if returned.contains(appid) {
                            progress.record_fetched(&[*appid]).await;
                        } else {
                            still_missing.push(*appid);
                        }
                    }
                }
                Err(error) if chunk.len() > 1 && !error.is_rate_limit() => {
                    warn!(?chunk, %error, "Steam GetItems failed for chunk; retrying its apps one at a time");
                    still_missing.extend_from_slice(chunk);
                }
                Err(error) => {
                    warn!(?chunk, %error, "Steam GetItems failed for chunk; leaving it for the next fetch");
                    progress.record_failed(chunk).await;
                }
            }

            if !progress.is_complete() {
                tokio::time::sleep(delay).await;
            }
        }

        let mut after_individual = Vec::new();
        for (index, &appid) in still_missing.iter().enumerate() {
            match client.fetch_store_items(&[appid]).await {
                Ok(items) => {
                    let items = Self::requested_items(&[appid], items);
                    if items.is_empty() {
                        after_individual.push(appid);
                    } else {
                        Self::upsert_items(state, &items).await?;
                        progress.record_fetched(&[appid]).await;
                    }
                }
                Err(error) if error.is_rate_limit() => {
                    let not_asked = &still_missing[index..];
                    warn!(?not_asked, %error, "Steam GetItems rate limited; leaving the remaining apps for the next fetch");
                    progress.record_failed(not_asked).await;
                    break;
                }
                Err(error) => {
                    warn!(appid, %error, "Steam GetItems failed for app; leaving it for the next fetch");
                    progress.record_failed(&[appid]).await;
                }
            }
            tokio::time::sleep(delay).await;
        }

        let mut after_details = Vec::new();
        for appid in after_individual {
            match client.fetch_app_details(appid).await {
                Ok(Some(detail)) => {
                    Self::upsert_from_app_details(state, appid, &detail).await?;
                    progress.record_fetched(&[appid]).await;
                }
                Ok(None) => after_details.push(appid),
                Err(error) => {
                    warn!(appid, %error, "Steam appdetails failed; leaving it for the next fetch");
                    progress.record_failed(&[appid]).await;
                }
            }
            tokio::time::sleep(delay).await;
        }

        for appid in after_details {
            Self::upsert_unavailable(state, appid).await?;
            progress.record_fetched(&[appid]).await;
        }

        info!(
            fetched = progress.fetched,
            failed = progress.failed.len(),
            "game cache updated from Steam"
        );
        Ok(progress.failed)
    }

    /// The items Steam returned for the `appids` asked for. Steam answers
    /// delisted or hidden apps with an item whose appid is 0.
    fn requested_items(appids: &[u32], items: Vec<StoreItem>) -> Vec<StoreItem> {
        items
            .into_iter()
            .filter(|item| appids.contains(&item.appid))
            .collect()
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
                        .collect()
                })
                .unwrap_or_default();

            let tags_json = serde_json::to_string(&tag_names)?;
            let raw_json = serde_json::to_string(item)?;
            let is_released = derive_is_released(item);

            Self::upsert_row(
                state,
                SteamGameUpsert {
                    appid,
                    name: &name,
                    short_description: &short_description,
                    header_image_url: &header_image_url,
                    tags_json: &tags_json,
                    raw_json: &raw_json,
                    is_released,
                    fetched_at: &now,
                },
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
        let header_image_url = header_image_url_from_app_details(detail).unwrap_or_else(|| {
            normalize_header_image_url(detail.header_image.as_deref().unwrap_or(""), appid)
        });
        let raw_json = serde_json::to_string(detail)?;
        let is_released = derive_is_released_from_app_details(detail);

        Self::upsert_row(
            state,
            SteamGameUpsert {
                appid,
                name: &name,
                short_description: &short_description,
                header_image_url: &header_image_url,
                tags_json: "[]",
                raw_json: &raw_json,
                is_released,
                fetched_at: &now,
            },
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
            SteamGameUpsert {
                appid,
                name: &name,
                short_description,
                header_image_url: &header_image_url,
                tags_json: "[]",
                raw_json: "{}",
                is_released: None,
                fetched_at: &now,
            },
        )
        .await
    }

    async fn upsert_row(state: &AppState, row: SteamGameUpsert<'_>) -> AppResult<()> {
        let SteamGameUpsert {
            appid,
            name,
            short_description,
            header_image_url,
            tags_json,
            raw_json,
            is_released,
            fetched_at,
        } = row;
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

/// The fields written to one `steam_games` row by `GameCache::upsert_row`.
struct SteamGameUpsert<'a> {
    appid: u32,
    name: &'a str,
    short_description: &'a str,
    header_image_url: &'a str,
    tags_json: &'a str,
    raw_json: &'a str,
    is_released: Option<bool>,
    fetched_at: &'a str,
}

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
        let appid = self.appid as u32;
        let header_image_url = serde_json::from_str::<StoreItem>(&self.raw_json)
            .ok()
            .map(|item| header_image_url_from_item(&item))
            .or_else(|| {
                serde_json::from_str::<AppDetailsData>(&self.raw_json)
                    .ok()
                    .and_then(|detail| header_image_url_from_app_details(&detail))
            })
            .unwrap_or_else(|| normalize_header_image_url(&self.header_image_url, appid));

        let is_released = serde_json::from_str::<StoreItem>(&self.raw_json)
            .ok()
            .and_then(|item| derive_is_released(&item))
            .or_else(|| {
                serde_json::from_str::<AppDetailsData>(&self.raw_json)
                    .ok()
                    .and_then(|detail| derive_is_released_from_app_details(&detail))
            })
            .or_else(|| self.is_released.map(|value| value != 0));

        let price = price_from_raw_json(&self.raw_json);
        let play_modes = play_modes_from_raw_json(&self.raw_json);

        GameInfo {
            appid: self.appid as u32,
            name: self.name,
            short_description: self.short_description,
            header_image_url,
            tags,
            is_released,
            price,
            play_modes,
        }
    }

    fn into_game_detail(self, appid: u32, tag_map: &HashMap<u32, String>) -> GameDetail {
        let store_item = serde_json::from_str::<StoreItem>(&self.raw_json).ok();
        let base = self.into_game_info();

        let tags = store_item
            .as_ref()
            .and_then(|item| item.tags.as_ref())
            .map(|tags_list| {
                tags_list
                    .iter()
                    .filter_map(|t| tag_map.get(&t.tagid).cloned())
                    .collect::<Vec<_>>()
            })
            .filter(|tags| !tags.is_empty())
            .unwrap_or_else(|| base.tags.clone());

        let full_description = store_item
            .as_ref()
            .and_then(|item| item.full_description_bbcode.clone())
            .filter(|value| !value.trim().is_empty());

        let screenshots = store_item
            .as_ref()
            .map(screenshot_urls_from_item)
            .unwrap_or_default();

        let trailers = store_item
            .as_ref()
            .and_then(|item| item.trailers.as_ref())
            .map(|items| {
                trailer_entries(items)
                    .into_iter()
                    .enumerate()
                    .map(|(index, entry)| GameTrailer {
                        name: entry.name,
                        thumbnail_url: entry.thumbnail_url,
                        video_url: trailer_proxy_video_url(appid, index, false),
                        video_webm_url: entry
                            .video_webm_url
                            .map(|_| trailer_proxy_video_url(appid, index, true)),
                        category: entry.category,
                    })
                    .collect()
            })
            .unwrap_or_default();

        GameDetail {
            appid: base.appid,
            name: base.name,
            short_description: base.short_description,
            full_description,
            header_image_url: base.header_image_url,
            tags,
            is_released: base.is_released,
            price: base.price,
            play_modes: base.play_modes,
            screenshots,
            trailers,
            store_url: format!("https://store.steampowered.com/app/{appid}"),
        }
    }
}

fn play_modes_from_raw_json(raw_json: &str) -> Option<GamePlayModes> {
    serde_json::from_str::<StoreItem>(raw_json)
        .ok()
        .and_then(|item| play_modes_from_item(&item))
}

fn price_from_raw_json(raw_json: &str) -> Option<GamePrice> {
    serde_json::from_str::<StoreItem>(raw_json)
        .ok()
        .and_then(|item| price_from_store_item(&item))
}

pub async fn load_games_map(db: &SqlitePool, appids: &[u32]) -> AppResult<HashMap<u32, GameInfo>> {
    let mut map = HashMap::new();
    for appid in appids {
        if let Some(game) = GameCache::load_game(db, *appid).await? {
            map.insert(*appid, game);
        }
    }
    Ok(map)
}

/// Progress through `GameCache::fetch_missing`'s walk, reported to its
/// `on_progress` callback. Apps Steam failed for count as done: they keep the
/// row they have (or stay uncached), and the next fetch tries them again.
struct WalkProgress<F, S> {
    total: i64,
    fetched: i64,
    /// The appids Steam failed for.
    failed: Vec<u32>,
    on_progress: F,
    on_settled: S,
}

/// Apps `GameCache::fetch_missing`'s walk is done with.
enum Settled<'a> {
    /// They have a fresh row: stored by the walk (as unavailable, too) or
    /// refreshed by someone else before it asked Steam.
    Cached(&'a [u32]),
    /// Steam failed for them; they keep the row they had, or stay uncached.
    Failed(&'a [u32]),
}

impl Settled<'_> {
    fn appids(&self) -> &[u32] {
        match self {
            Settled::Cached(appids) | Settled::Failed(appids) => appids,
        }
    }
}

impl<F, Fut, S> WalkProgress<F, S>
where
    F: FnMut(i64, i64) -> Fut,
    Fut: std::future::Future<Output = ()>,
    S: FnMut(Settled<'_>),
{
    fn new(total: i64, on_progress: F, on_settled: S) -> Self {
        Self {
            total,
            fetched: 0,
            failed: Vec::new(),
            on_progress,
            on_settled,
        }
    }

    fn done(&self) -> i64 {
        self.fetched + self.failed.len() as i64
    }

    fn is_complete(&self) -> bool {
        self.done() >= self.total
    }

    /// Records `appids` as stored, once their rows are written.
    async fn record_fetched(&mut self, appids: &[u32]) {
        self.fetched += appids.len() as i64;
        (self.on_settled)(Settled::Cached(appids));
        self.report().await;
    }

    async fn record_failed(&mut self, appids: &[u32]) {
        self.failed.extend_from_slice(appids);
        (self.on_settled)(Settled::Failed(appids));
        self.report().await;
    }

    async fn report(&mut self) {
        let done = self.done();
        (self.on_progress)(done, self.total).await;
    }
}

use sqlx::Row;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use sqlx::sqlite::SqlitePoolOptions;
    use tokio::sync::RwLock;

    use crate::config::Config;
    use crate::jobs::registry::SyncJobRegistry;
    use crate::steam::fetch_cooldown::{ManualClock, FAILED_FETCH_COOLDOWN};
    use crate::steam::store::fake::{FakeSteam, SteamGate};
    use crate::steam::{FetchCooldown, GameRefreshQueue};

    const PRICE_TTL_HOURS: i64 = 24;
    const GAME_TTL_DAYS: i64 = 7;

    const COMING_SOON_WITHOUT_PRICE: u32 = 1001;
    const WITHOUT_CATEGORIES: u32 = 1002;
    const STEAM_STUB: u32 = 1003;
    const UNAVAILABLE: u32 = 1004;
    const FROM_APP_DETAILS: u32 = 1005;
    const ALL_FIXTURE_APPIDS: [u32; 5] = [
        COMING_SOON_WITHOUT_PRICE,
        WITHOUT_CATEGORIES,
        STEAM_STUB,
        UNAVAILABLE,
        FROM_APP_DETAILS,
    ];

    async fn test_db() -> SqlitePool {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&db).await.unwrap();
        db
    }

    /// An `AppState` over `db` with no delay between Steam calls.
    /// A test state whose cooldown is measured with the returned clock.
    async fn test_state_with_clock() -> (AppState, ManualClock) {
        let mut state = test_state(test_db().await);
        let clock = ManualClock::new();
        state.fetch_cooldown = Arc::new(FetchCooldown::with_clock(clock.clone()));
        (state, clock)
    }

    fn test_state(db: SqlitePool) -> AppState {
        AppState {
            db,
            http: reqwest::Client::new(),
            tags: Arc::new(RwLock::new(HashMap::new())),
            sync_jobs: Arc::new(SyncJobRegistry::new()),
            game_refresh: Arc::new(GameRefreshQueue::new()),
            fetch_cooldown: Arc::new(FetchCooldown::new()),
            config: Config {
                database_url: "sqlite::memory:".into(),
                bind_addr: "127.0.0.1:0".into(),
                game_cache_ttl_days: GAME_TTL_DAYS,
                price_cache_ttl_hours: PRICE_TTL_HOURS,
                steam_chunk_size: 50,
                steam_chunk_delay_ms: 0,
                steam_api_key: None,
            },
        }
    }

    async fn fetch_missing(
        state: &AppState,
        steam: &FakeSteam,
        appids: &[u32],
    ) -> AppResult<Vec<u32>> {
        GameCache::fetch_missing(state, steam, appids, |_, _| async {}).await
    }

    async fn stored_raw_json(db: &SqlitePool, appid: u32) -> Option<String> {
        GameCache::load_row(db, appid)
            .await
            .unwrap()
            .map(|row| row.raw_json)
    }

    async fn stored_name(db: &SqlitePool, appid: u32) -> Option<String> {
        GameCache::load_game(db, appid)
            .await
            .unwrap()
            .map(|game| game.name)
    }

    async fn insert_game(
        db: &SqlitePool,
        appid: u32,
        raw_json: &str,
        is_released: Option<bool>,
        fetched_at: DateTime<Utc>,
    ) {
        sqlx::query(
            "INSERT INTO steam_games (appid, name, raw_json, is_released, fetched_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(appid as i64)
        .bind(format!("Steam app {appid}"))
        .bind(raw_json)
        .bind(is_released.map(i64::from))
        .bind(fetched_at.to_rfc3339())
        .execute(db)
        .await
        .unwrap();
    }

    /// The kinds of row Steam leaves "incomplete" no matter how often we ask.
    async fn insert_rows_steam_never_completes(db: &SqlitePool, fetched_at: DateTime<Utc>) {
        insert_game(
            db,
            COMING_SOON_WITHOUT_PRICE,
            &format!(r#"{{"appid":{COMING_SOON_WITHOUT_PRICE},"name":"Soon","is_coming_soon":true,"categories":{{"supported_player_categoryids":[2]}}}}"#),
            Some(false),
            fetched_at,
        )
        .await;
        insert_game(
            db,
            WITHOUT_CATEGORIES,
            &format!(r#"{{"appid":{WITHOUT_CATEGORIES},"name":"No modes","is_coming_soon":false,"best_purchase_option":{{"formatted_final_price":"$9.99"}}}}"#),
            Some(true),
            fetched_at,
        )
        .await;
        insert_game(
            db,
            STEAM_STUB,
            &format!(r#"{{"appid":{STEAM_STUB},"name":"","is_coming_soon":null,"basic_info":null,"best_purchase_option":null,"categories":null}}"#),
            None,
            fetched_at,
        )
        .await;
        insert_game(db, UNAVAILABLE, "{}", None, fetched_at).await;
        insert_game(
            db,
            FROM_APP_DETAILS,
            &format!(r#"{{"name":"Old detail","steam_appid":{FROM_APP_DETAILS},"short_description":null,"header_image":null,"coming_soon":false}}"#),
            Some(true),
            fetched_at,
        )
        .await;
    }

    #[tokio::test]
    async fn recently_fetched_rows_are_not_price_stale_even_when_steam_returned_no_price_or_categories(
    ) {
        let db = test_db().await;
        insert_rows_steam_never_completes(&db, Utc::now()).await;

        let stale = GameCache::get_price_stale_appids(&db, &ALL_FIXTURE_APPIDS, PRICE_TTL_HOURS)
            .await
            .unwrap();

        assert_eq!(stale, HashSet::new());
    }

    #[tokio::test]
    async fn recently_fetched_rows_count_as_cached_even_when_release_status_is_unknown() {
        let db = test_db().await;
        insert_rows_steam_never_completes(&db, Utc::now()).await;

        let cached = GameCache::get_cached_appids(&db, &ALL_FIXTURE_APPIDS, GAME_TTL_DAYS)
            .await
            .unwrap();

        assert_eq!(cached, HashSet::from(ALL_FIXTURE_APPIDS));
    }

    #[tokio::test]
    async fn rows_older_than_the_price_ttl_are_price_stale_but_still_cached() {
        let db = test_db().await;
        insert_rows_steam_never_completes(&db, Utc::now() - Duration::hours(PRICE_TTL_HOURS + 1))
            .await;

        let stale = GameCache::get_price_stale_appids(&db, &ALL_FIXTURE_APPIDS, PRICE_TTL_HOURS)
            .await
            .unwrap();
        let cached = GameCache::get_cached_appids(&db, &ALL_FIXTURE_APPIDS, GAME_TTL_DAYS)
            .await
            .unwrap();

        assert_eq!(stale, HashSet::from(ALL_FIXTURE_APPIDS));
        assert_eq!(cached, HashSet::from(ALL_FIXTURE_APPIDS));
    }

    #[tokio::test]
    async fn rows_older_than_the_game_ttl_are_not_cached() {
        let db = test_db().await;
        insert_rows_steam_never_completes(&db, Utc::now() - Duration::days(GAME_TTL_DAYS + 1))
            .await;

        let cached = GameCache::get_cached_appids(&db, &ALL_FIXTURE_APPIDS, GAME_TTL_DAYS)
            .await
            .unwrap();

        assert_eq!(cached, HashSet::new());
    }

    const LISTED: u32 = 2001;
    const HIDDEN: u32 = 2002;

    #[tokio::test]
    async fn an_app_steam_answers_with_appid_0_is_stored_as_unavailable_and_appid_0_is_not_stored()
    {
        let state = test_state(test_db().await);
        let steam = FakeSteam::new()
            .with_store_item(LISTED, "Listed")
            .with_hidden_app(HIDDEN);

        fetch_missing(&state, &steam, &[LISTED, HIDDEN])
            .await
            .unwrap();

        assert_eq!(
            stored_raw_json(&state.db, HIDDEN).await.as_deref(),
            Some("{}")
        );
        assert_eq!(stored_raw_json(&state.db, 0).await, None);
        assert_eq!(
            stored_name(&state.db, LISTED).await.as_deref(),
            Some("Listed")
        );
    }

    const ONLY_IN_APP_DETAILS: u32 = 2003;

    #[tokio::test]
    async fn an_app_only_appdetails_knows_is_stored_from_app_details() {
        let state = test_state(test_db().await);
        let steam = FakeSteam::new().with_app_details(ONLY_IN_APP_DETAILS, "Details only");

        fetch_missing(&state, &steam, &[ONLY_IN_APP_DETAILS])
            .await
            .unwrap();

        assert_eq!(
            stored_name(&state.db, ONLY_IN_APP_DETAILS).await.as_deref(),
            Some("Details only")
        );
        assert_ne!(
            stored_raw_json(&state.db, ONLY_IN_APP_DETAILS)
                .await
                .as_deref(),
            Some("{}")
        );
    }

    const FAILING: u32 = 2004;
    const FAILING_RAW_JSON: &str = r#"{"appid":2004,"name":"Old row"}"#;

    /// A row for `FAILING` old enough to be fetched again.
    async fn insert_price_stale_failing_row(db: &SqlitePool) {
        insert_game(
            db,
            FAILING,
            FAILING_RAW_JSON,
            Some(true),
            Utc::now() - Duration::hours(PRICE_TTL_HOURS + 1),
        )
        .await;
    }

    async fn assert_failing_row_unchanged_and_still_due(db: &SqlitePool) {
        assert_eq!(
            stored_raw_json(db, FAILING).await.as_deref(),
            Some(FAILING_RAW_JSON)
        );
        let stale = GameCache::get_price_stale_appids(db, &[FAILING], PRICE_TTL_HOURS)
            .await
            .unwrap();
        assert_eq!(stale, HashSet::from([FAILING]));
    }

    #[tokio::test]
    async fn a_chunk_steam_fails_for_is_skipped_and_the_other_chunks_are_cached() {
        let mut state = test_state(test_db().await);
        state.config.steam_chunk_size = 1;
        insert_price_stale_failing_row(&state.db).await;
        let steam = FakeSteam::new()
            .with_store_item(LISTED, "Listed")
            .with_store_item(FAILING, "New name")
            .failing_every_call_for(FAILING);

        fetch_missing(&state, &steam, &[FAILING, LISTED])
            .await
            .unwrap();

        assert_failing_row_unchanged_and_still_due(&state.db).await;
        assert_eq!(
            stored_name(&state.db, LISTED).await.as_deref(),
            Some("Listed")
        );
    }

    #[tokio::test]
    async fn an_app_appdetails_fails_for_keeps_its_row_instead_of_becoming_unavailable() {
        let state = test_state(test_db().await);
        insert_price_stale_failing_row(&state.db).await;
        let steam = FakeSteam::new()
            .with_store_item(LISTED, "Listed")
            .failing_app_details_for(FAILING);

        fetch_missing(&state, &steam, &[FAILING, LISTED])
            .await
            .unwrap();

        assert_failing_row_unchanged_and_still_due(&state.db).await;
        assert_eq!(
            stored_name(&state.db, LISTED).await.as_deref(),
            Some("Listed")
        );
    }

    #[tokio::test]
    async fn an_app_the_single_app_retry_fails_for_keeps_its_row_instead_of_becoming_unavailable() {
        let state = test_state(test_db().await);
        insert_price_stale_failing_row(&state.db).await;
        let steam = FakeSteam::new()
            .with_store_item(LISTED, "Listed")
            .failing_when_asked_alone_for(FAILING);

        fetch_missing(&state, &steam, &[FAILING, LISTED])
            .await
            .unwrap();

        assert_failing_row_unchanged_and_still_due(&state.db).await;
        assert_eq!(
            stored_name(&state.db, LISTED).await.as_deref(),
            Some("Listed")
        );
    }

    #[tokio::test]
    async fn an_uncached_app_steam_fails_for_stays_uncached_instead_of_becoming_unavailable() {
        let state = test_state(test_db().await);
        let steam = FakeSteam::new()
            .with_store_item(LISTED, "Listed")
            .failing_app_details_for(FAILING);

        fetch_missing(&state, &steam, &[FAILING, LISTED])
            .await
            .unwrap();

        assert_eq!(stored_raw_json(&state.db, FAILING).await, None);
        assert_eq!(
            stored_name(&state.db, LISTED).await.as_deref(),
            Some("Listed")
        );
    }

    #[tokio::test]
    async fn an_app_steam_keeps_failing_for_does_not_hold_back_the_rest_of_its_chunk() {
        let state = test_state(test_db().await);
        let steam = FakeSteam::new()
            .with_store_item(LISTED, "Listed")
            .with_app_details(ONLY_IN_APP_DETAILS, "Details only")
            .failing_every_call_for(FAILING);

        let failed = fetch_missing(&state, &steam, &[LISTED, FAILING, ONLY_IN_APP_DETAILS])
            .await
            .unwrap();

        assert_eq!(failed, vec![FAILING]);
        assert_eq!(
            stored_name(&state.db, LISTED).await.as_deref(),
            Some("Listed")
        );
        assert_eq!(
            stored_name(&state.db, ONLY_IN_APP_DETAILS).await.as_deref(),
            Some("Details only")
        );
        assert_eq!(stored_raw_json(&state.db, FAILING).await, None);
    }

    #[tokio::test]
    async fn a_rate_limited_chunk_fails_every_app_in_it_without_asking_for_them_one_at_a_time() {
        let state = test_state(test_db().await);
        let steam = FakeSteam::new()
            .with_store_item(LISTED, "Listed")
            .with_app_details(ONLY_IN_APP_DETAILS, "Details only")
            .rate_limiting_every_store_items_call();

        let failed = fetch_missing(&state, &steam, &[LISTED, ONLY_IN_APP_DETAILS])
            .await
            .unwrap();

        assert_eq!(failed, vec![LISTED, ONLY_IN_APP_DETAILS]);
        assert_eq!(steam.calls(), 1);
        assert_eq!(stored_raw_json(&state.db, LISTED).await, None);
    }

    #[tokio::test]
    async fn a_rate_limit_while_retrying_a_failed_chunk_fails_the_apps_not_asked_for_yet() {
        let state = test_state(test_db().await);
        let steam = FakeSteam::new()
            .with_store_item(LISTED, "Listed")
            .with_app_details(ONLY_IN_APP_DETAILS, "Details only")
            .failing_every_call_for(FAILING)
            .rate_limiting_store_items_calls_for_one_app();

        let failed = fetch_missing(&state, &steam, &[LISTED, FAILING, ONLY_IN_APP_DETAILS])
            .await
            .unwrap();

        assert_eq!(failed, vec![LISTED, FAILING, ONLY_IN_APP_DETAILS]);
        assert_eq!(steam.calls(), 2);
    }

    const STALE: u32 = 3001;
    const STALE_RAW_JSON: &str = r#"{"appid":3001,"name":"Old name"}"#;

    async fn insert_price_stale_row(db: &SqlitePool, appid: u32, raw_json: &str) {
        insert_game(
            db,
            appid,
            raw_json,
            Some(true),
            Utc::now() - Duration::hours(PRICE_TTL_HOURS + 1),
        )
        .await;
    }

    /// Moves an existing row's `fetched_at` back past the price TTL.
    async fn make_price_stale(db: &SqlitePool, appid: u32) {
        sqlx::query("UPDATE steam_games SET fetched_at = ? WHERE appid = ?")
            .bind((Utc::now() - Duration::hours(PRICE_TTL_HOURS + 1)).to_rfc3339())
            .bind(appid as i64)
            .execute(db)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn ensure_cached_serves_a_price_stale_row_as_is_and_queues_it_for_refresh() {
        let state = test_state(test_db().await);
        insert_price_stale_row(&state.db, STALE, STALE_RAW_JSON).await;
        let steam = FakeSteam::new().with_store_item(STALE, "New name");

        let _ = GameCache::ensure_cached_from(&state, &steam, &[STALE])
            .await
            .unwrap();

        assert_eq!(
            stored_raw_json(&state.db, STALE).await.as_deref(),
            Some(STALE_RAW_JSON)
        );
        assert_eq!(state.game_refresh.queued(), vec![STALE]);
    }

    const UNCACHED: u32 = 3002;

    #[tokio::test]
    async fn ensure_cached_waits_for_an_app_with_no_row_and_queues_nothing() {
        let state = test_state(test_db().await);
        let steam = FakeSteam::new().with_store_item(UNCACHED, "Uncached");

        let _ = GameCache::ensure_cached_from(&state, &steam, &[UNCACHED])
            .await
            .unwrap();

        assert_eq!(
            stored_name(&state.db, UNCACHED).await.as_deref(),
            Some("Uncached")
        );
        assert_eq!(state.game_refresh.queued(), Vec::<u32>::new());
    }

    #[tokio::test]
    async fn a_stale_app_already_queued_is_not_queued_again_or_given_a_second_worker() {
        let state = test_state(test_db().await);
        insert_price_stale_row(&state.db, STALE, STALE_RAW_JSON).await;
        let steam = FakeSteam::new().with_store_item(STALE, "New name");

        let first = GameCache::ensure_cached_from(&state, &steam, &[STALE])
            .await
            .unwrap();
        let second = GameCache::ensure_cached_from(&state, &steam, &[STALE])
            .await
            .unwrap();

        assert!(first, "the first call starts the refresh worker");
        assert!(!second, "the second call leaves it to the running worker");
        assert_eq!(state.game_refresh.queued(), vec![STALE]);
    }

    #[tokio::test]
    async fn the_background_refresh_refetches_queued_apps_and_empties_the_queue() {
        let state = test_state(test_db().await);
        insert_price_stale_row(&state.db, STALE, STALE_RAW_JSON).await;
        let steam = FakeSteam::new().with_store_item(STALE, "New name");
        let _ = GameCache::ensure_cached_from(&state, &steam, &[STALE])
            .await
            .unwrap();

        GameCache::refresh_queued(&state, &steam).await;

        assert_eq!(
            stored_name(&state.db, STALE).await.as_deref(),
            Some("New name")
        );
        assert_eq!(state.game_refresh.queued(), Vec::<u32>::new());
        let stale = GameCache::get_price_stale_appids(&state.db, &[STALE], PRICE_TTL_HOURS)
            .await
            .unwrap();
        assert_eq!(stale, HashSet::new());
    }

    const STALE_LATER: u32 = 3003;

    #[tokio::test]
    async fn a_stale_app_queued_after_the_worker_drained_the_queue_starts_a_new_worker() {
        let state = test_state(test_db().await);
        insert_price_stale_row(&state.db, STALE, STALE_RAW_JSON).await;
        let steam = FakeSteam::new()
            .with_store_item(STALE, "New name")
            .with_store_item(STALE_LATER, "Later");
        let _ = GameCache::ensure_cached_from(&state, &steam, &[STALE])
            .await
            .unwrap();
        GameCache::refresh_queued(&state, &steam).await;
        insert_price_stale_row(&state.db, STALE_LATER, r#"{"appid":3003,"name":"Old"}"#).await;

        let starts_worker = GameCache::ensure_cached_from(&state, &steam, &[STALE, STALE_LATER])
            .await
            .unwrap();

        assert!(starts_worker);
        assert_eq!(state.game_refresh.queued(), vec![STALE_LATER]);
    }

    #[tokio::test]
    async fn ensure_cached_serves_a_row_past_only_the_game_ttl_as_is_and_queues_it_for_refresh() {
        let mut state = test_state(test_db().await);
        state.config.price_cache_ttl_hours = (GAME_TTL_DAYS + 30) * 24;
        insert_game(
            &state.db,
            STALE,
            STALE_RAW_JSON,
            Some(true),
            Utc::now() - Duration::days(GAME_TTL_DAYS + 1),
        )
        .await;
        let steam = FakeSteam::new().with_store_item(STALE, "New name");

        let _ = GameCache::ensure_cached_from(&state, &steam, &[STALE])
            .await
            .unwrap();

        assert_eq!(
            stored_raw_json(&state.db, STALE).await.as_deref(),
            Some(STALE_RAW_JSON)
        );
        assert_eq!(state.game_refresh.queued(), vec![STALE]);
    }

    #[tokio::test]
    async fn a_stale_app_steam_fails_for_is_not_queued_again_during_the_cooldown() {
        let state = test_state(test_db().await);
        insert_price_stale_failing_row(&state.db).await;
        let steam = FakeSteam::new().failing_every_call_for(FAILING);
        let _ = GameCache::ensure_cached_from(&state, &steam, &[FAILING])
            .await
            .unwrap();
        GameCache::refresh_queued(&state, &steam).await;

        let starts_worker = GameCache::ensure_cached_from(&state, &steam, &[FAILING])
            .await
            .unwrap();

        assert!(!starts_worker);
        assert_eq!(state.game_refresh.queued(), Vec::<u32>::new());
        assert_failing_row_unchanged_and_still_due(&state.db).await;
    }

    #[tokio::test]
    async fn a_stale_app_steam_failed_for_is_queued_again_once_the_cooldown_has_passed() {
        let (state, clock) = test_state_with_clock().await;
        insert_price_stale_failing_row(&state.db).await;
        let steam = FakeSteam::new().failing_every_call_for(FAILING);
        let _ = GameCache::ensure_cached_from(&state, &steam, &[FAILING])
            .await
            .unwrap();
        GameCache::refresh_queued(&state, &steam).await;

        clock.advance(FAILED_FETCH_COOLDOWN);
        let starts_worker = GameCache::ensure_cached_from(&state, &steam, &[FAILING])
            .await
            .unwrap();

        assert!(starts_worker);
        assert_eq!(state.game_refresh.queued(), vec![FAILING]);
    }

    #[tokio::test]
    async fn an_app_refreshed_in_the_same_batch_as_a_failing_one_is_not_held_back() {
        let mut state = test_state(test_db().await);
        state.config.steam_chunk_size = 1;
        insert_price_stale_failing_row(&state.db).await;
        insert_price_stale_row(&state.db, STALE, STALE_RAW_JSON).await;
        let steam = FakeSteam::new()
            .with_store_item(STALE, "New name")
            .failing_every_call_for(FAILING);
        let _ = GameCache::ensure_cached_from(&state, &steam, &[FAILING, STALE])
            .await
            .unwrap();
        GameCache::refresh_queued(&state, &steam).await;
        assert_eq!(
            stored_name(&state.db, STALE).await.as_deref(),
            Some("New name")
        );

        make_price_stale(&state.db, STALE).await;
        let starts_worker = GameCache::ensure_cached_from(&state, &steam, &[FAILING, STALE])
            .await
            .unwrap();

        assert!(starts_worker);
        assert_eq!(state.game_refresh.queued(), vec![STALE]);
    }

    #[tokio::test]
    async fn a_stale_app_whose_refresh_cannot_be_stored_is_not_queued_again_during_the_cooldown() {
        let state = test_state(test_db().await);
        insert_price_stale_row(&state.db, STALE, STALE_RAW_JSON).await;
        sqlx::query(
            "CREATE TRIGGER steam_games_read_only BEFORE UPDATE ON steam_games \
             BEGIN SELECT RAISE(ABORT, 'database or disk is full'); END",
        )
        .execute(&state.db)
        .await
        .unwrap();
        let steam = FakeSteam::new().with_store_item(STALE, "New name");
        let _ = GameCache::ensure_cached_from(&state, &steam, &[STALE])
            .await
            .unwrap();
        GameCache::refresh_queued(&state, &steam).await;

        let starts_worker = GameCache::ensure_cached_from(&state, &steam, &[STALE])
            .await
            .unwrap();

        assert!(!starts_worker);
        assert_eq!(state.game_refresh.queued(), Vec::<u32>::new());
    }

    /// `future`'s output; fails the test when it hangs, as a list load left
    /// waiting for a claim that is never released would.
    async fn finishes<T>(future: impl std::future::Future<Output = T>) -> T {
        tokio::time::timeout(std::time::Duration::from_secs(5), future)
            .await
            .expect("list loads hung")
    }

    /// Waits until a call has reached `steam`.
    async fn reached(steam: &FakeSteam) {
        while steam.calls() == 0 {
            tokio::task::yield_now().await;
        }
    }

    /// Waits until a list load waits for another list load's fetch of
    /// `appid`, or, should it not wait, until it has reached `steam` too.
    async fn overlapped(state: &AppState, steam: &FakeSteam, appid: u32) {
        while state.game_refresh.waiting_for(appid) == 0 && steam.calls() < 2 {
            tokio::task::yield_now().await;
        }
    }

    /// Lets `steam`'s held calls answer once a second list load has
    /// overlapped the first one's fetch of `appid`.
    async fn release_after_overlap(state: &AppState, steam: &FakeSteam, gate: &SteamGate) {
        overlapped(state, steam, UNCACHED).await;
        gate.release();
    }

    #[tokio::test]
    async fn overlapping_list_loads_fetch_an_uncached_app_from_steam_once() {
        let state = test_state(test_db().await);
        let (steam, gate) = FakeSteam::new()
            .with_store_item(UNCACHED, "Uncached")
            .held();

        let (first, second, ()) = finishes(async {
            tokio::join!(
                GameCache::ensure_cached_from(&state, &steam, &[UNCACHED]),
                GameCache::ensure_cached_from(&state, &steam, &[UNCACHED]),
                release_after_overlap(&state, &steam, &gate),
            )
        })
        .await;

        let _ = first.unwrap();
        let _ = second.unwrap();
        assert_eq!(steam.calls(), 1);
        assert_eq!(
            stored_name(&state.db, UNCACHED).await.as_deref(),
            Some("Uncached")
        );
    }

    #[tokio::test]
    async fn overlapping_list_loads_ask_a_failing_steam_for_an_uncached_app_once() {
        let state = test_state(test_db().await);
        let (steam, gate) = FakeSteam::new().failing_every_call_for(UNCACHED).held();

        let (first, second, ()) = finishes(async {
            tokio::join!(
                GameCache::ensure_cached_from(&state, &steam, &[UNCACHED]),
                GameCache::ensure_cached_from(&state, &steam, &[UNCACHED]),
                release_after_overlap(&state, &steam, &gate),
            )
        })
        .await;

        let _ = first.unwrap();
        let _ = second.unwrap();
        assert_eq!(steam.calls(), 1);
        assert_eq!(stored_raw_json(&state.db, UNCACHED).await, None);
    }

    #[tokio::test]
    async fn a_dropped_list_load_does_not_leave_its_uncached_app_claimed() {
        let state = test_state(test_db().await);
        let (held, _gate) = FakeSteam::new()
            .with_store_item(UNCACHED, "Uncached")
            .held();
        tokio::select! {
            _ = GameCache::ensure_cached_from(&state, &held, &[UNCACHED]) => {
                panic!("held Steam answered")
            }
            () = reached(&held) => {}
        }

        let answering = FakeSteam::new().with_store_item(UNCACHED, "Uncached");
        let _ = finishes(GameCache::ensure_cached_from(
            &state,
            &answering,
            &[UNCACHED],
        ))
        .await
        .unwrap();

        assert_eq!(answering.calls(), 1);
        assert_eq!(
            stored_name(&state.db, UNCACHED).await.as_deref(),
            Some("Uncached")
        );
    }

    #[tokio::test]
    async fn overlapping_list_loads_retry_a_failed_chunk_one_app_at_a_time_once() {
        let state = test_state(test_db().await);
        let (steam, gate) = FakeSteam::new()
            .with_store_item(UNCACHED, "Uncached")
            .failing_every_call_for(FAILING)
            .held();

        let (first, second, ()) = finishes(async {
            tokio::join!(
                GameCache::ensure_cached_from(&state, &steam, &[UNCACHED, FAILING]),
                GameCache::ensure_cached_from(&state, &steam, &[UNCACHED, FAILING]),
                release_after_overlap(&state, &steam, &gate),
            )
        })
        .await;

        let _ = first.unwrap();
        let _ = second.unwrap();
        // The failed chunk, then each of its apps on its own.
        assert_eq!(steam.calls(), 3);
        assert_eq!(
            stored_name(&state.db, UNCACHED).await.as_deref(),
            Some("Uncached")
        );
        assert_eq!(stored_raw_json(&state.db, FAILING).await, None);
    }

    #[tokio::test]
    async fn a_list_load_waiting_for_a_dropped_list_loads_fetch_fetches_the_app_itself() {
        let state = test_state(test_db().await);
        let (held, _gate) = FakeSteam::new()
            .with_store_item(UNCACHED, "Uncached")
            .held();
        let mut first = Box::pin(GameCache::ensure_cached_from(&state, &held, &[UNCACHED]));
        tokio::select! {
            _ = &mut first => panic!("held Steam answered"),
            () = reached(&held) => {}
        }

        let answering = FakeSteam::new().with_store_item(UNCACHED, "Uncached");
        let (second, ()) = finishes(async {
            tokio::join!(
                GameCache::ensure_cached_from(&state, &answering, &[UNCACHED]),
                async {
                    overlapped(&state, &answering, UNCACHED).await;
                    drop(first);
                },
            )
        })
        .await;

        let _ = second.unwrap();
        assert_eq!(answering.calls(), 1);
        assert_eq!(
            stored_name(&state.db, UNCACHED).await.as_deref(),
            Some("Uncached")
        );
    }

    const IN_FIRST_CHUNK: u32 = 3004;
    const IN_SECOND_CHUNK: u32 = 3005;

    /// Waits until a list load waits for another list load's fetch of `appid`.
    async fn waiting_for(state: &AppState, appid: u32) {
        while state.game_refresh.waiting_for(appid) == 0 {
            tokio::task::yield_now().await;
        }
    }

    #[tokio::test]
    async fn a_list_load_waiting_for_another_loads_fetch_returns_once_its_app_is_stored() {
        let mut state = test_state(test_db().await);
        state.config.steam_chunk_size = 1;
        let (steam, gate) = FakeSteam::new()
            .with_store_item(IN_FIRST_CHUNK, "First")
            .with_store_item(IN_SECOND_CHUNK, "Second")
            .held();

        let (first, second) = finishes(async {
            tokio::join!(
                GameCache::ensure_cached_from(&state, &steam, &[IN_FIRST_CHUNK, IN_SECOND_CHUNK]),
                async {
                    reached(&steam).await;
                    let (second, ()) = tokio::join!(
                        GameCache::ensure_cached_from(&state, &steam, &[IN_FIRST_CHUNK]),
                        async {
                            waiting_for(&state, IN_FIRST_CHUNK).await;
                            gate.answer_one();
                        },
                    );
                    // The first load's second chunk is still held.
                    let stored = stored_name(&state.db, IN_FIRST_CHUNK).await;
                    let second_chunk = stored_name(&state.db, IN_SECOND_CHUNK).await;
                    gate.release();
                    (second, stored, second_chunk)
                },
            )
        })
        .await;

        let _ = first.unwrap();
        let (second, stored, second_chunk) = second;
        let _ = second.unwrap();
        assert_eq!(stored.as_deref(), Some("First"));
        assert_eq!(second_chunk, None);
        // One call per chunk: the waiting load did not fetch the app again.
        assert_eq!(steam.calls(), 2);
    }

    #[tokio::test]
    async fn a_list_load_waiting_for_another_loads_fetch_of_an_app_steam_fails_for_does_not_fetch_it_again(
    ) {
        let mut state = test_state(test_db().await);
        state.config.steam_chunk_size = 1;
        let (steam, gate) = FakeSteam::new()
            .failing_every_call_for(IN_FIRST_CHUNK)
            .with_store_item(IN_SECOND_CHUNK, "Second")
            .held();

        let (first, second) = finishes(async {
            tokio::join!(
                GameCache::ensure_cached_from(&state, &steam, &[IN_FIRST_CHUNK, IN_SECOND_CHUNK]),
                async {
                    reached(&steam).await;
                    let (second, ()) = tokio::join!(
                        GameCache::ensure_cached_from(&state, &steam, &[IN_FIRST_CHUNK]),
                        async {
                            waiting_for(&state, IN_FIRST_CHUNK).await;
                            gate.answer_one();
                        },
                    );
                    // The first load's second chunk is still held.
                    let second_chunk = stored_name(&state.db, IN_SECOND_CHUNK).await;
                    gate.release();
                    (second, second_chunk)
                },
            )
        })
        .await;

        let _ = first.unwrap();
        let (second, second_chunk) = second;
        let _ = second.unwrap();
        assert_eq!(second_chunk, None);
        // One call per chunk: the waiting load did not ask for the app again.
        assert_eq!(steam.calls(), 2);
        assert_eq!(stored_raw_json(&state.db, IN_FIRST_CHUNK).await, None);
    }

    #[tokio::test]
    async fn an_uncached_app_steam_fails_for_is_not_fetched_again_during_the_cooldown() {
        let state = test_state(test_db().await);
        let failing = FakeSteam::new().failing_every_call_for(UNCACHED);
        let _ = GameCache::ensure_cached_from(&state, &failing, &[UNCACHED])
            .await
            .unwrap();

        let answering = FakeSteam::new().with_store_item(UNCACHED, "Uncached");
        let _ = GameCache::ensure_cached_from(&state, &answering, &[UNCACHED])
            .await
            .unwrap();

        assert_eq!(stored_raw_json(&state.db, UNCACHED).await, None);
    }

    #[tokio::test]
    async fn an_uncached_app_steam_failed_for_is_fetched_again_once_the_cooldown_has_passed() {
        let (state, clock) = test_state_with_clock().await;
        let failing = FakeSteam::new().failing_every_call_for(UNCACHED);
        let _ = GameCache::ensure_cached_from(&state, &failing, &[UNCACHED])
            .await
            .unwrap();

        let answering = FakeSteam::new().with_store_item(UNCACHED, "Uncached");
        clock.advance(FAILED_FETCH_COOLDOWN - std::time::Duration::from_secs(1));
        let _ = GameCache::ensure_cached_from(&state, &answering, &[UNCACHED])
            .await
            .unwrap();
        assert_eq!(stored_raw_json(&state.db, UNCACHED).await, None);

        clock.advance(std::time::Duration::from_secs(1));
        let _ = GameCache::ensure_cached_from(&state, &answering, &[UNCACHED])
            .await
            .unwrap();

        assert_eq!(
            stored_name(&state.db, UNCACHED).await.as_deref(),
            Some("Uncached")
        );
    }
}
