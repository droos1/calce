use std::fmt;
use std::sync::Arc;

use crate::domain::trade::Trade;
use crate::services::market_data::InMemoryMarketDataService;

/// Bundles trades with the market data needed to run calculations.
///
/// Built by `DataLoader` in production or manually in tests.
/// Uses `Arc` so in-memory backends can share data without cloning.
pub struct CalcInputs {
    pub trades: Vec<Trade>,
    pub market_data: Arc<InMemoryMarketDataService>,
}

impl fmt::Debug for CalcInputs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CalcInputs")
            .field("trades", &self.trades.len())
            .finish_non_exhaustive()
    }
}
