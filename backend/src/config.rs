use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub game_cache_ttl_days: i64,
    pub price_cache_ttl_hours: i64,
    pub steam_chunk_size: usize,
    pub steam_chunk_delay_ms: u64,
    pub steam_api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            // Keeps its pre-Squadlist file name on purpose so existing deployments keep their data.
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:playtogether.db?mode=rwc".into()),
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            game_cache_ttl_days: env::var("GAME_CACHE_TTL_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(7),
            price_cache_ttl_hours: env::var("PRICE_CACHE_TTL_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
            steam_chunk_size: env::var("STEAM_CHUNK_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50),
            steam_chunk_delay_ms: env::var("STEAM_CHUNK_DELAY_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(200),
            steam_api_key: env::var("STEAM_API_KEY").ok().filter(|k| !k.is_empty()),
        }
    }
}
