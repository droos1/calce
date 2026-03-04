use std::sync::Arc;

use calce_core::services::market_data::InMemoryMarketDataService;
use calce_core::services::user_data::InMemoryUserDataService;

#[derive(Clone)]
pub struct AppState {
    pub market_data: Arc<InMemoryMarketDataService>,
    pub user_data: Arc<InMemoryUserDataService>,
}
