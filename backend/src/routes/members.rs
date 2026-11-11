use axum::{
    extract::{Path, State},
    Json,
};

use crate::db;
use crate::error::{AppError, AppResult};
use crate::jobs::sync_member;
use crate::middleware::auth::AuthMember;
use crate::models::{MeResponse, SetSelectionRequest, SyncStatus, UpdateMemberRequest};
use crate::state::AppState;

pub async fn get_me(
    State(state): State<AppState>,
    auth: AuthMember,
) -> AppResult<Json<MeResponse>> {
    let member = db::get_member_by_token(&state.db, &auth.token)
        .await?
        .into_member()?;
    let group = db::get_group(&state.db, auth.group_id)
        .await?
        .into_group()?;

    Ok(Json(MeResponse { member, group }))
}

pub async fn update_me(
    State(state): State<AppState>,
    auth: AuthMember,
    Json(body): Json<UpdateMemberRequest>,
) -> AppResult<Json<MeResponse>> {
    let mut steam_changed = false;

    if let Some(name) = &body.display_name {
        if name.trim().is_empty() {
            return Err(AppError::BadRequest("display_name cannot be empty".into()));
        }
        sqlx::query("UPDATE members SET display_name = ? WHERE id = ?")
            .bind(name.trim())
            .bind(auth.id.to_string())
            .execute(&state.db)
            .await?;
    }

    if let Some(steam_id) = body.steam_id {
        sqlx::query("UPDATE members SET steam_id = ? WHERE id = ?")
            .bind(steam_id as i64)
            .bind(auth.id.to_string())
            .execute(&state.db)
            .await?;
        steam_changed = true;
    }

    if steam_changed {
        sync_member::trigger_sync(&state, auth.id).await?;
    }

    get_me(State(state), auth).await
}

pub async fn refresh_wishlist(
    State(state): State<AppState>,
    auth: AuthMember,
) -> AppResult<Json<SyncStatus>> {
    sync_member::trigger_sync(&state, auth.id).await?;
    let status = db::get_sync_status(&state.db, auth.id).await?;
    Ok(Json(status))
}

pub async fn get_sync_status(
    State(state): State<AppState>,
    auth: AuthMember,
) -> AppResult<Json<SyncStatus>> {
    let status = db::get_sync_status(&state.db, auth.id).await?;
    Ok(Json(status))
}

pub async fn set_selection(
    State(state): State<AppState>,
    auth: AuthMember,
    Path(appid): Path<u32>,
    Json(body): Json<SetSelectionRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT appid FROM wishlist_items WHERE member_id = ? AND appid = ?
         UNION
         SELECT appid FROM owned_games WHERE member_id = ? AND appid = ?",
    )
    .bind(auth.id.to_string())
    .bind(appid as i64)
    .bind(auth.id.to_string())
    .bind(appid as i64)
    .fetch_optional(&state.db)
    .await?;

    if exists.is_none() {
        return Err(AppError::BadRequest(
            "appid not in your wishlist or library".into(),
        ));
    }

    let wants = if body.wants_to_play { 1 } else { 0 };

    sqlx::query(
        "INSERT INTO member_selections (member_id, appid, wants_to_play) VALUES (?, ?, ?)
         ON CONFLICT(member_id, appid) DO UPDATE SET wants_to_play = excluded.wants_to_play",
    )
    .bind(auth.id.to_string())
    .bind(appid as i64)
    .bind(wants)
    .execute(&state.db)
    .await?;

    Ok(Json(
        serde_json::json!({ "appid": appid, "wants_to_play": body.wants_to_play }),
    ))
}

pub async fn leave_group(
    State(state): State<AppState>,
    auth: AuthMember,
) -> AppResult<Json<serde_json::Value>> {
    sync_member::cancel_member_sync(&state, auth.id).await;

    let member_id = auth.id.to_string();
    let group_id = auth.group_id.to_string();

    let member_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM members WHERE group_id = ?")
        .bind(&group_id)
        .fetch_one(&state.db)
        .await?;

    let mut tx = state.db.begin().await?;

    delete_member_data(&mut tx, &member_id).await?;

    sqlx::query("DELETE FROM members WHERE id = ?")
        .bind(&member_id)
        .execute(&mut *tx)
        .await?;

    if member_count == 1 {
        let deleted = sqlx::query("DELETE FROM play_groups WHERE id = ?")
            .bind(&group_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();

        if deleted == 0 {
            return Err(AppError::NotFound);
        }
    }

    tx.commit().await?;

    Ok(Json(serde_json::json!({ "left": true })))
}

async fn delete_member_data(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    member_id: &str,
) -> AppResult<()> {
    sqlx::query("DELETE FROM group_game_votes WHERE member_id = ?")
        .bind(member_id)
        .execute(&mut **tx)
        .await?;

    sqlx::query("DELETE FROM member_selections WHERE member_id = ?")
        .bind(member_id)
        .execute(&mut **tx)
        .await?;

    sqlx::query("DELETE FROM wishlist_items WHERE member_id = ?")
        .bind(member_id)
        .execute(&mut **tx)
        .await?;

    sqlx::query("DELETE FROM owned_games WHERE member_id = ?")
        .bind(member_id)
        .execute(&mut **tx)
        .await?;

    sqlx::query("DELETE FROM member_sync_jobs WHERE member_id = ?")
        .bind(member_id)
        .execute(&mut **tx)
        .await?;

    Ok(())
}
