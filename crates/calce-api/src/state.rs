use std::sync::Arc;

use calce_data::market_data_store::MarketDataStore;
use calce_data::user_data_store::UserDataStore;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub market_data: Arc<MarketDataStore>,
    pub user_data: Arc<UserDataStore>,
    pub pool: Option<PgPool>,
}
