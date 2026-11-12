use std::collections::HashMap;
use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::jobs::registry::SyncJobRegistry;
use crate::steam::{FetchCooldown, GameRefreshQueue};

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub http: reqwest::Client,
    pub tags: Arc<RwLock<HashMap<u32, String>>>,
    pub sync_jobs: Arc<SyncJobRegistry>,
    pub game_refresh: Arc<GameRefreshQueue>,
    pub fetch_cooldown: Arc<FetchCooldown>,
    pub config: Config,
}
