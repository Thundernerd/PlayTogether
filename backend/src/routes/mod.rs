mod games;
mod groups;
mod members;
mod wishlists;

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/groups", post(groups::create_group))
        .route("/api/groups/by-invite/{code}", get(groups::get_by_invite))
        .route("/api/groups/by-invite/{code}/join", post(groups::join_group))
        .route("/api/groups/{group_id}", delete(groups::delete_group))
        .route("/api/members/me", get(members::get_me).patch(members::update_me))
        .route("/api/members/me/refresh-wishlist", post(members::refresh_wishlist))
        .route("/api/members/me/sync-status", get(members::get_sync_status))
        .route(
            "/api/members/me/selections/{appid}",
            put(members::set_selection),
        )
        .route(
            "/api/groups/{group_id}/wishlists",
            get(wishlists::get_group_wishlists),
        )
        .route(
            "/api/groups/{group_id}/shared-games",
            get(wishlists::get_shared_games),
        )
        .route(
            "/api/groups/{group_id}/games/{appid}/vote",
            put(wishlists::set_vote),
        )
        .route(
            "/api/groups/{group_id}/games/{appid}/rating",
            put(wishlists::set_rating),
        )
        .route("/api/games/{appid}", get(games::get_game_detail))
        .route(
            "/api/games/{appid}/trailers/{index}/video",
            get(games::stream_trailer_video),
        )
}
