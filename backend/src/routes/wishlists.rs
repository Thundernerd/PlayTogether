use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthMember;
use crate::models::{
    MemberLibrary, MemberSelectionInfo, MemberWishlist, OwnedGameItemView, SetRatingRequest,
    SetVoteRequest, SharedGame, WishlistItemView,
};
use crate::state::AppState;
use crate::steam::{load_games_map, GameCache};

pub async fn get_group_wishlists(
    State(state): State<AppState>,
    auth: AuthMember,
    Path(group_id): Path<Uuid>,
) -> AppResult<Json<Vec<MemberWishlist>>> {
    ensure_group_access(&auth, group_id)?;

    let members = db::list_group_members(&state.db, group_id).await?;
    let mut result = Vec::new();

    for member in members {
        let rows = sqlx::query_as::<_, WishlistRow>(
            "SELECT wi.appid, wi.priority, wi.date_added,
                    COALESCE(ms.wants_to_play, 0) as wants_to_play
             FROM wishlist_items wi
             LEFT JOIN member_selections ms ON ms.member_id = wi.member_id AND ms.appid = wi.appid
             WHERE wi.member_id = ?
             ORDER BY wi.priority ASC",
        )
        .bind(member.id.to_string())
        .fetch_all(&state.db)
        .await?;

        let appids: Vec<u32> = rows.iter().map(|r| r.appid as u32).collect();
        GameCache::ensure_cached(&state, &appids).await?;
        let games = load_games_map(&state.db, &appids).await?;

        let items: Vec<WishlistItemView> = rows
            .into_iter()
            .map(|row| WishlistItemView {
                appid: row.appid as u32,
                priority: row.priority,
                date_added: row.date_added,
                game: games.get(&(row.appid as u32)).cloned(),
                wants_to_play: row.wants_to_play != 0,
            })
            .collect();

        result.push(MemberWishlist { member, items });
    }

    Ok(Json(result))
}

pub async fn get_group_libraries(
    State(state): State<AppState>,
    auth: AuthMember,
    Path(group_id): Path<Uuid>,
) -> AppResult<Json<Vec<MemberLibrary>>> {
    ensure_group_access(&auth, group_id)?;

    let members = db::list_group_members(&state.db, group_id).await?;
    let mut result = Vec::new();

    for member in members {
        let rows = sqlx::query_as::<_, OwnedGameRow>(
            "SELECT og.appid, og.playtime_forever, og.rtime_last_played,
                    COALESCE(ms.wants_to_play, 0) as wants_to_play
             FROM owned_games og
             LEFT JOIN member_selections ms ON ms.member_id = og.member_id AND ms.appid = og.appid
             WHERE og.member_id = ?
             ORDER BY og.playtime_forever DESC",
        )
        .bind(member.id.to_string())
        .fetch_all(&state.db)
        .await?;

        let appids: Vec<u32> = rows.iter().map(|r| r.appid as u32).collect();
        GameCache::ensure_cached(&state, &appids).await?;
        let games = load_games_map(&state.db, &appids).await?;

        let items: Vec<OwnedGameItemView> = rows
            .into_iter()
            .map(|row| OwnedGameItemView {
                appid: row.appid as u32,
                playtime_forever: row.playtime_forever,
                rtime_last_played: row.rtime_last_played,
                game: games.get(&(row.appid as u32)).cloned(),
                wants_to_play: row.wants_to_play != 0,
            })
            .collect();

        result.push(MemberLibrary { member, items });
    }

    Ok(Json(result))
}

pub async fn get_shared_games(
    State(state): State<AppState>,
    auth: AuthMember,
    Path(group_id): Path<Uuid>,
) -> AppResult<Json<Vec<SharedGame>>> {
    ensure_group_access(&auth, group_id)?;

    let rows = sqlx::query_as::<_, SharedAppRow>(
        "SELECT ms.appid, m.id as member_id, m.display_name
         FROM member_selections ms
         JOIN members m ON m.id = ms.member_id
         WHERE m.group_id = ? AND ms.wants_to_play = 1
         ORDER BY ms.appid",
    )
    .bind(group_id.to_string())
    .fetch_all(&state.db)
    .await?;

    let mut by_appid: std::collections::BTreeMap<u32, Vec<MemberSelectionInfo>> =
        std::collections::BTreeMap::new();

    for row in rows {
        by_appid
            .entry(row.appid as u32)
            .or_default()
            .push(MemberSelectionInfo {
                member_id: row.member_id,
                display_name: row.display_name,
            });
    }

    let appids: Vec<u32> = by_appid.keys().copied().collect();
    GameCache::ensure_cached(&state, &appids).await?;
    let games = load_games_map(&state.db, &appids).await?;

    let mut shared = Vec::new();

    for (appid, selected_by) in by_appid {
        let Some(game) = games.get(&appid).cloned() else {
            continue;
        };

        let vote_row = sqlx::query_as::<_, VoteAggRow>(
            "SELECT
               SUM(CASE WHEN vote = 1 THEN 1 ELSE 0 END) as upvotes,
               SUM(CASE WHEN vote = -1 THEN 1 ELSE 0 END) as downvotes,
               AVG(CASE WHEN star_rating IS NOT NULL THEN star_rating END) as avg_stars
             FROM group_game_votes
             WHERE group_id = ? AND appid = ?",
        )
        .bind(group_id.to_string())
        .bind(appid as i64)
        .fetch_one(&state.db)
        .await?;

        let my_row = sqlx::query_as::<_, MyVoteRow>(
            "SELECT vote, star_rating FROM group_game_votes WHERE group_id = ? AND appid = ? AND member_id = ?",
        )
        .bind(group_id.to_string())
        .bind(appid as i64)
        .bind(auth.id.to_string())
        .fetch_optional(&state.db)
        .await?;

        let upvotes = vote_row.upvotes.unwrap_or(0);
        let downvotes = vote_row.downvotes.unwrap_or(0);

        shared.push(SharedGame {
            appid,
            game,
            selected_by,
            upvotes,
            downvotes,
            net_score: upvotes - downvotes,
            avg_stars: vote_row.avg_stars,
            my_vote: my_row.as_ref().map(|r| r.vote).unwrap_or(0),
            my_star_rating: my_row.and_then(|r| r.star_rating),
        });
    }

    shared.sort_by(|a, b| {
        b.net_score
            .cmp(&a.net_score)
            .then_with(|| {
                b.avg_stars
                    .unwrap_or(0.0)
                    .partial_cmp(&a.avg_stars.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.game.name.cmp(&b.game.name))
    });

    Ok(Json(shared))
}

pub async fn set_vote(
    State(state): State<AppState>,
    auth: AuthMember,
    Path((group_id, appid)): Path<(Uuid, u32)>,
    Json(body): Json<SetVoteRequest>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_group_access(&auth, group_id)?;
    ensure_shared_game(&state, group_id, appid).await?;

    let vote = match body.vote.as_str() {
        "up" => 1,
        "down" => -1,
        "none" => 0,
        _ => {
            return Err(AppError::BadRequest(
                "vote must be up, down, or none".into(),
            ))
        }
    };

    sqlx::query(
        "INSERT INTO group_game_votes (group_id, appid, member_id, vote)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(group_id, appid, member_id) DO UPDATE SET vote = excluded.vote",
    )
    .bind(group_id.to_string())
    .bind(appid as i64)
    .bind(auth.id.to_string())
    .bind(vote)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "appid": appid, "vote": vote })))
}

pub async fn set_rating(
    State(state): State<AppState>,
    auth: AuthMember,
    Path((group_id, appid)): Path<(Uuid, u32)>,
    Json(body): Json<SetRatingRequest>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_group_access(&auth, group_id)?;
    ensure_shared_game(&state, group_id, appid).await?;

    if let Some(stars) = body.stars {
        if !(1..=5).contains(&stars) {
            return Err(AppError::BadRequest("stars must be between 1 and 5".into()));
        }
    }

    sqlx::query(
        "INSERT INTO group_game_votes (group_id, appid, member_id, star_rating)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(group_id, appid, member_id) DO UPDATE SET star_rating = excluded.star_rating",
    )
    .bind(group_id.to_string())
    .bind(appid as i64)
    .bind(auth.id.to_string())
    .bind(body.stars)
    .execute(&state.db)
    .await?;

    Ok(Json(
        serde_json::json!({ "appid": appid, "stars": body.stars }),
    ))
}

fn ensure_group_access(auth: &AuthMember, group_id: Uuid) -> AppResult<()> {
    if auth.group_id != group_id {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

async fn ensure_shared_game(state: &AppState, group_id: Uuid, appid: u32) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM member_selections ms
         JOIN members m ON m.id = ms.member_id
         WHERE m.group_id = ? AND ms.appid = ? AND ms.wants_to_play = 1",
    )
    .bind(group_id.to_string())
    .bind(appid as i64)
    .fetch_one(&state.db)
    .await?;

    if count == 0 {
        return Err(AppError::BadRequest(
            "game is not in the shared list".into(),
        ));
    }
    Ok(())
}

struct WishlistRow {
    appid: i64,
    priority: i64,
    date_added: i64,
    wants_to_play: i64,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for WishlistRow {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            appid: row.try_get("appid")?,
            priority: row.try_get("priority")?,
            date_added: row.try_get("date_added")?,
            wants_to_play: row.try_get("wants_to_play")?,
        })
    }
}

struct OwnedGameRow {
    appid: i64,
    playtime_forever: i64,
    rtime_last_played: Option<i64>,
    wants_to_play: i64,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for OwnedGameRow {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            appid: row.try_get("appid")?,
            playtime_forever: row.try_get("playtime_forever")?,
            rtime_last_played: row.try_get("rtime_last_played")?,
            wants_to_play: row.try_get("wants_to_play")?,
        })
    }
}

struct SharedAppRow {
    appid: i64,
    member_id: Uuid,
    display_name: String,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for SharedAppRow {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            appid: row.try_get("appid")?,
            member_id: Uuid::parse_str(row.try_get::<String, _>("member_id")?.as_str())
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            display_name: row.try_get("display_name")?,
        })
    }
}

struct VoteAggRow {
    upvotes: Option<i64>,
    downvotes: Option<i64>,
    avg_stars: Option<f64>,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for VoteAggRow {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            upvotes: row.try_get("upvotes")?,
            downvotes: row.try_get("downvotes")?,
            avg_stars: row.try_get("avg_stars")?,
        })
    }
}

struct MyVoteRow {
    vote: i64,
    star_rating: Option<i64>,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for MyVoteRow {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            vote: row.try_get("vote")?,
            star_rating: row.try_get("star_rating")?,
        })
    }
}

use sqlx::Row;
