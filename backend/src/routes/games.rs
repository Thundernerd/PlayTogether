use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, header},
    response::Response,
};
use futures_util::StreamExt;
use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::models::GameDetail;
use crate::state::AppState;
use crate::steam::GameCache;

#[derive(Deserialize)]
pub(crate) struct TrailerVideoQuery {
    format: Option<String>,
}

pub async fn get_game_detail(
    State(state): State<AppState>,
    Path(appid): Path<u32>,
) -> AppResult<Json<GameDetail>> {
    GameCache::ensure_detail_cached(&state, appid).await?;

    let detail = GameCache::load_game_detail(&state, appid)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(detail))
}

pub async fn stream_trailer_video(
    State(state): State<AppState>,
    Path((appid, index)): Path<(u32, usize)>,
    Query(query): Query<TrailerVideoQuery>,
    headers: HeaderMap,
) -> AppResult<Response> {
    GameCache::ensure_detail_cached(&state, appid).await?;

    let webm = query.format.as_deref() == Some("webm");
    let cdn_url = GameCache::trailer_cdn_url(&state.db, appid, index, webm).await?;

    let mut request = state.http.get(&cdn_url);
    if let Some(range) = headers.get(header::RANGE) {
        request = request.header(header::RANGE, range);
    }

    let upstream = request.send().await?;
    let status = upstream.status();

    let mut builder = Response::builder().status(status);
    for name in [
        header::CONTENT_TYPE,
        header::CONTENT_LENGTH,
        header::CONTENT_RANGE,
        header::ACCEPT_RANGES,
    ] {
        if let Some(value) = upstream.headers().get(&name) {
            builder = builder.header(name, value);
        }
    }

    if !upstream.headers().contains_key(header::CONTENT_TYPE) {
        let content_type = if webm { "video/webm" } else { "video/mp4" };
        builder = builder.header(header::CONTENT_TYPE, content_type);
    }

    let stream = upstream.bytes_stream().map(|chunk| {
        chunk.map_err(std::io::Error::other)
    });

    builder
        .body(Body::from_stream(stream))
        .map_err(|err| AppError::Internal(err.to_string()))
}
