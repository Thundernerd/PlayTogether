use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::db::{self, generate_invite_code};
use crate::error::{AppError, AppResult};
use crate::jobs::sync_member;
use crate::middleware::auth::AuthMember;
use crate::models::{
    CreateGroupRequest, CreateGroupResponse, GroupDetailResponse, JoinGroupRequest,
    JoinGroupResponse,
};
use crate::state::AppState;

pub async fn create_group(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(body): Json<CreateGroupRequest>,
) -> AppResult<Json<CreateGroupResponse>> {
    if body.name.trim().is_empty() || body.display_name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "name and display_name are required".into(),
        ));
    }

    let group_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let token = Uuid::new_v4().to_string();
    let invite_code = generate_invite_code();
    let now = Utc::now().to_rfc3339();

    let mut tx = state.db.begin().await?;

    sqlx::query("INSERT INTO play_groups (id, name, invite_code, created_at) VALUES (?, ?, ?, ?)")
        .bind(group_id.to_string())
        .bind(body.name.trim())
        .bind(&invite_code)
        .bind(&now)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO members (id, group_id, display_name, steam_id, token, created_at) VALUES (?, ?, ?, NULL, ?, ?)",
    )
    .bind(member_id.to_string())
    .bind(group_id.to_string())
    .bind(body.display_name.trim())
    .bind(&token)
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO member_sync_jobs (member_id, status, updated_at) VALUES (?, 'idle', ?)",
    )
    .bind(member_id.to_string())
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let group = db::get_group(&state.db, group_id).await?.into_group()?;
    let member = db::get_member_by_token(&state.db, &token)
        .await?
        .into_member()?;

    Ok(Json(CreateGroupResponse {
        group,
        member,
        token,
    }))
}

pub async fn get_by_invite(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(code): Path<String>,
) -> AppResult<Json<GroupDetailResponse>> {
    let group_row = db::get_group_by_invite(&state.db, &code).await?;
    let group_id = Uuid::parse_str(&group_row.id).map_err(|e| AppError::Internal(e.to_string()))?;
    let group = group_row.into_group()?;
    let members = db::list_group_members(&state.db, group_id).await?;

    Ok(Json(GroupDetailResponse { group, members }))
}

pub async fn join_group(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(code): Path<String>,
    Json(body): Json<JoinGroupRequest>,
) -> AppResult<Json<JoinGroupResponse>> {
    if body.display_name.trim().is_empty() {
        return Err(AppError::BadRequest("display_name is required".into()));
    }

    let group_row = db::get_group_by_invite(&state.db, &code).await?;
    let group_id = Uuid::parse_str(&group_row.id).map_err(|e| AppError::Internal(e.to_string()))?;

    let member_id = Uuid::new_v4();
    let token = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO members (id, group_id, display_name, steam_id, token, created_at) VALUES (?, ?, ?, NULL, ?, ?)",
    )
    .bind(member_id.to_string())
    .bind(group_id.to_string())
    .bind(body.display_name.trim())
    .bind(&token)
    .bind(&now)
    .execute(&state.db)
    .await?;

    sqlx::query(
        "INSERT INTO member_sync_jobs (member_id, status, updated_at) VALUES (?, 'idle', ?)",
    )
    .bind(member_id.to_string())
    .bind(&now)
    .execute(&state.db)
    .await?;

    let group = group_row.into_group()?;
    let member = db::get_member_by_token(&state.db, &token)
        .await?
        .into_member()?;

    Ok(Json(JoinGroupResponse {
        group,
        member,
        token,
    }))
}

pub async fn delete_group(
    State(state): State<AppState>,
    auth: AuthMember,
    Path(group_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.group_id != group_id {
        return Err(AppError::Forbidden);
    }

    sync_member::cancel_group_jobs(&state, group_id).await?;

    let group_id_str = group_id.to_string();
    let mut tx = state.db.begin().await?;

    sqlx::query("DELETE FROM group_game_votes WHERE group_id = ?")
        .bind(&group_id_str)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "DELETE FROM member_selections WHERE member_id IN (SELECT id FROM members WHERE group_id = ?)",
    )
    .bind(&group_id_str)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM wishlist_items WHERE member_id IN (SELECT id FROM members WHERE group_id = ?)",
    )
    .bind(&group_id_str)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM owned_games WHERE member_id IN (SELECT id FROM members WHERE group_id = ?)",
    )
    .bind(&group_id_str)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM member_sync_jobs WHERE member_id IN (SELECT id FROM members WHERE group_id = ?)",
    )
    .bind(&group_id_str)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM members WHERE group_id = ?")
        .bind(&group_id_str)
        .execute(&mut *tx)
        .await?;

    let deleted = sqlx::query("DELETE FROM play_groups WHERE id = ?")
        .bind(&group_id_str)
        .execute(&mut *tx)
        .await?
        .rows_affected();

    if deleted == 0 {
        return Err(AppError::NotFound);
    }

    tx.commit().await?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
