use uuid::Uuid;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::steam::{GameCache, SteamClient};

pub async fn spawn_member_sync(state: AppState, member_id: Uuid, steam_id: u64) {
    let registry = state.sync_jobs.clone();
    let task_state = state.clone();

    let handle = tokio::spawn(async move {
        let result = run_sync(&task_state, member_id, steam_id).await;
        registry.remove(member_id).await;

        if let Err(e) = result {
            if is_task_cancelled(&e) {
                tracing::info!(%member_id, "member sync cancelled");
                return;
            }
            tracing::error!(%member_id, error = %e, "member sync failed");
            if db::member_exists(&task_state.db, member_id)
                .await
                .unwrap_or(false)
            {
                let _ = db::upsert_sync_job(
                    &task_state.db,
                    member_id,
                    "error",
                    None,
                    0,
                    0,
                    Some(&e.to_string()),
                )
                .await;
            }
        }
    });

    state.sync_jobs.insert(member_id, handle).await;
}

pub async fn cancel_group_jobs(state: &AppState, group_id: Uuid) -> AppResult<Vec<Uuid>> {
    let member_ids = db::list_member_ids(&state.db, group_id).await?;
    state.sync_jobs.cancel_many(&member_ids).await;
    Ok(member_ids)
}

pub async fn cancel_member_sync(state: &AppState, member_id: Uuid) {
    state.sync_jobs.cancel(member_id).await;
}

async fn run_sync(state: &AppState, member_id: Uuid, steam_id: u64) -> AppResult<()> {
    ensure_member_active(state, member_id).await?;

    let client = SteamClient::new(state.http.clone());
    let mut appids: Vec<u32> = Vec::new();

    if let Some(api_key) = &state.config.steam_api_key {
        db::upsert_sync_job(
            &state.db,
            member_id,
            "running",
            Some("fetching_owned"),
            0,
            0,
            None,
        )
        .await?;

        ensure_member_active(state, member_id).await?;

        let owned = client.fetch_owned_games(api_key, steam_id).await?;

        sqlx::query("DELETE FROM owned_games WHERE member_id = ?")
            .bind(member_id.to_string())
            .execute(&state.db)
            .await?;

        for item in &owned {
            sqlx::query(
                "INSERT INTO owned_games (member_id, appid, playtime_forever, rtime_last_played) VALUES (?, ?, ?, ?)",
            )
            .bind(member_id.to_string())
            .bind(item.appid as i64)
            .bind(item.playtime_forever.unwrap_or(0))
            .bind(item.rtime_last_played)
            .execute(&state.db)
            .await?;
            appids.push(item.appid);
        }
    } else {
        tracing::warn!(%member_id, "STEAM_API_KEY not set; skipping owned games sync");
    }

    db::upsert_sync_job(
        &state.db,
        member_id,
        "running",
        Some("fetching_wishlist"),
        0,
        0,
        None,
    )
    .await?;

    let items = client.fetch_wishlist(steam_id).await?;

    ensure_member_active(state, member_id).await?;

    sqlx::query("DELETE FROM wishlist_items WHERE member_id = ?")
        .bind(member_id.to_string())
        .execute(&state.db)
        .await?;

    for item in &items {
        sqlx::query(
            "INSERT INTO wishlist_items (member_id, appid, priority, date_added) VALUES (?, ?, ?, ?)",
        )
        .bind(member_id.to_string())
        .bind(item.appid as i64)
        .bind(item.priority)
        .bind(item.date_added)
        .execute(&state.db)
        .await?;
        appids.push(item.appid);
    }

    appids.sort_unstable();
    appids.dedup();

    let appids = appids;

    db::upsert_sync_job(
        &state.db,
        member_id,
        "running",
        Some("fetching_games"),
        0,
        0,
        None,
    )
    .await?;

    GameCache::fetch_missing(state, &client, &appids, |done, total| {
        let db = state.db.clone();
        async move {
            if !db::member_exists(&db, member_id).await.unwrap_or(false) {
                return;
            }
            let _ = db::upsert_sync_job(
                &db,
                member_id,
                "running",
                Some("fetching_games"),
                done,
                total,
                None,
            )
            .await;
        }
    })
    .await?;

    ensure_member_active(state, member_id).await?;

    db::upsert_sync_job(&state.db, member_id, "complete", None, 0, 0, None).await?;

    Ok(())
}

pub async fn trigger_sync(state: &AppState, member_id: Uuid) -> AppResult<()> {
    let row = sqlx::query_scalar::<_, Option<i64>>("SELECT steam_id FROM members WHERE id = ?")
        .bind(member_id.to_string())
        .fetch_one(&state.db)
        .await?;

    let steam_id = row
        .ok_or_else(|| AppError::BadRequest("steam_id must be set before syncing".into()))?
        as u64;

    let status = db::get_sync_status(&state.db, member_id).await?;
    if matches!(status.status, crate::models::SyncJobStatus::Running) {
        return Err(AppError::Conflict("sync already in progress".into()));
    }

    spawn_member_sync(state.clone(), member_id, steam_id).await;
    Ok(())
}

async fn ensure_member_active(state: &AppState, member_id: Uuid) -> AppResult<()> {
    if db::member_exists(&state.db, member_id).await? {
        Ok(())
    } else {
        Err(AppError::Internal("sync cancelled".into()))
    }
}

fn is_task_cancelled(err: &AppError) -> bool {
    matches!(err, AppError::Internal(msg) if msg == "sync cancelled")
}
