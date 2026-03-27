use std::sync::Arc;

use calce_data::auth::AuthConfig;
use calce_data::auth::api_key::ApiKeyCache;
use calce_data::market_data_store::MarketDataStore;
use calce_data::user_data_store::UserDataStore;
use sqlx::PgPool;

use crate::rate_limit::KeyedRateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub market_data: Arc<MarketDataStore>,
    pub user_data: Arc<UserDataStore>,
    pub pool: Option<PgPool>,
    pub auth_config: AuthConfig,
    pub api_key_cache: ApiKeyCache,
    pub auth_rate_limiter: Arc<KeyedRateLimiter>,
}

impl AppState {
    pub fn require_pool(&self) -> Result<&PgPool, crate::error::ApiError> {
        self.pool
            .as_ref()
            .ok_or_else(|| crate::error::ApiError::BadRequest("database required".into()))
    }
}
