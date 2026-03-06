use crate::domain::trade::Trade;
use crate::services::market_data::InMemoryMarketDataService;

/// Bundles a user's trades with the market data needed to run calculations.
///
/// Built by `DataLoader` in production or manually in tests.
pub struct PortfolioData {
    pub trades: Vec<Trade>,
    pub market_data: InMemoryMarketDataService,
}
