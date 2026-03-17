use crate::error::DataResult;
use crate::in_memory_market_data::InMemoryMarketDataService;
use crate::market_data_store::{InstrumentSummary, MarketDataStore};
use crate::queries::market_data::MarketDataRepo;
use crate::queries::user_data::UserDataRepo;
use crate::user_data_store::{UserDataStore, UserSummary};
use sqlx::PgPool;

/// Bulk-load all data from Postgres into memory at startup.
///
/// # Errors
///
/// Propagates database errors.
pub async fn load_from_postgres(pool: &PgPool) -> DataResult<(MarketDataStore, UserDataStore)> {
    let md_repo = MarketDataRepo::new(pool.clone());
    let ud_repo = UserDataRepo::new(pool.clone());

    let (users_raw, instruments_raw, trades, all_prices, all_fx_rates) = tokio::try_join!(
        ud_repo.list_users_with_trade_counts(),
        md_repo.list_instruments(),
        ud_repo.get_all_trades(),
        md_repo.get_all_prices(),
        md_repo.get_all_fx_rates(),
    )?;

    let users: Vec<UserSummary> = users_raw
        .into_iter()
        .map(|(id, email, trade_count)| UserSummary {
            id,
            email,
            trade_count,
        })
        .collect();

    let instruments: Vec<InstrumentSummary> = instruments_raw
        .into_iter()
        .map(|(id, currency, name)| InstrumentSummary { id, currency, name })
        .collect();

    let mut md = InMemoryMarketDataService::new();
    for (instrument, date, price) in all_prices {
        md.add_price(&instrument, date, price);
    }
    for (date, rate) in all_fx_rates {
        md.add_fx_rate(rate, date);
    }
    md.freeze();

    let mut ud = UserDataStore::new();
    for trade in trades {
        ud.add_trade(trade);
    }
    ud.set_users(users);

    tracing::info!(
        "Data loaded: {} users, {} instruments, {} prices, {} FX rates",
        ud.user_count(),
        instruments.len(),
        md.price_count(),
        md.fx_rate_count(),
    );

    let market_store = MarketDataStore::from_parts(md, instruments);

    Ok((market_store, ud))
}
