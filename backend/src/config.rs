use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub cors_origin: String,
    pub game_cache_ttl_days: i64,
    pub steam_chunk_size: usize,
    pub steam_chunk_delay_ms: u64,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:playtogether.db?mode=rwc".into()),
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            cors_origin: env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into()),
            game_cache_ttl_days: env::var("GAME_CACHE_TTL_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(7),
            steam_chunk_size: env::var("STEAM_CHUNK_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50),
            steam_chunk_delay_ms: env::var("STEAM_CHUNK_DELAY_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(200),
        }
    }
}
