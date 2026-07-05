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

use axum::http::{HeaderName, HeaderValue, Method};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::jobs::registry::SyncJobRegistry;
use crate::state::AppState;
use crate::steam::TagCache;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "playwishlist_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let http = reqwest::Client::builder()
        .user_agent("PlayWishlist/0.1")
        .build()?;

    let state = AppState {
        db,
        http,
        tags: Arc::new(RwLock::new(HashMap::new())),
        sync_jobs: Arc::new(SyncJobRegistry::new()),
        config: config.clone(),
    };

    TagCache::warm(&state).await?;

    let cors = CorsLayer::new()
        .allow_origin(config.cors_origin.parse::<HeaderValue>()?)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            HeaderName::from_static("x-member-token"),
        ]);

    let app = routes::router()
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("listening on {}", config.bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
