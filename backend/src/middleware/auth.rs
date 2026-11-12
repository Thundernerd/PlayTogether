use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
};

use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub const MEMBER_TOKEN_HEADER: &str = "x-member-token";

#[derive(Clone, Debug)]
pub struct AuthMember {
    pub token: String,
    pub id: uuid::Uuid,
    pub group_id: uuid::Uuid,
}

impl FromRequestParts<AppState> for AuthMember {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token(&parts.headers)?;
        let member = db::get_member_by_token(&state.db, &token).await?;
        Ok(AuthMember {
            token,
            id: member.id,
            group_id: member.group_id,
        })
    }
}

fn extract_token(headers: &HeaderMap) -> AppResult<String> {
    headers
        .get(MEMBER_TOKEN_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .ok_or(AppError::Unauthorized)
}
