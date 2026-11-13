mod config;
mod db;
mod error;
mod jobs;
mod middleware;
mod models;
mod routes;
mod state;
mod steam;

use std::collections::HashMap;
use std::sync::Arc;

use sqlx::sqlite::SqlitePoolOptions;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::jobs::registry::SyncJobRegistry;
use crate::state::AppState;
use crate::steam::{FetchCooldown, GameCache, GameRefreshQueue, TagCache};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                concat!(env!("CARGO_CRATE_NAME"), "=debug,tower_http=debug").into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let config = Config::from_env();

    if config.steam_api_key.is_some() {
        tracing::info!("STEAM_API_KEY configured — owned-games library sync enabled");
    } else {
        tracing::warn!("STEAM_API_KEY not set — wishlist sync works, but library sync is skipped");
    }

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let http = reqwest::Client::builder()
        .user_agent("Squadlist/0.1")
        .build()?;

    let state = AppState {
        db,
        http,
        tags: Arc::new(RwLock::new(HashMap::new())),
        sync_jobs: Arc::new(SyncJobRegistry::new()),
        game_refresh: Arc::new(GameRefreshQueue::new()),
        fetch_cooldown: Arc::new(FetchCooldown::new()),
        config: config.clone(),
    };

    TagCache::warm(&state).await?;
    GameCache::backfill_header_urls(&state.db).await?;

    let app = routes::router()
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("listening on {}", config.bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
