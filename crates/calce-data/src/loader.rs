use serde::Serialize;

use calce_core::auth::SecurityContext;
use calce_core::domain::currency::Currency;
use calce_core::domain::instrument::InstrumentId;
use calce_core::domain::price::Price;
use calce_core::domain::trade::Trade;
use calce_core::domain::user::UserId;
use calce_core::error::{CalceError, CalceResult};
use calce_core::permissions;
use calce_core::services::market_data::{InMemoryMarketDataService, MarketDataService};
use calce_core::services::user_data::InMemoryUserDataService;
use calce_core::snapshot::PortfolioData;
use chrono::NaiveDate;

use crate::repo::market_data::MarketDataRepo;
use crate::repo::user_data::UserDataRepo;

pub struct DateRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

enum Backend {
    Postgres {
        market_data_repo: MarketDataRepo,
        user_data_repo: UserDataRepo,
    },
    InMemory {
        market_data: Box<InMemoryMarketDataService>,
        user_data: InMemoryUserDataService,
    },
}

/// Generic async data loader with no knowledge of which calculations exist.
///
/// In production, wraps Postgres repos. In tests, wraps in-memory services.
///
/// User-scoped methods require a `SecurityContext` and enforce access checks
/// via [`permissions::can_access_user_data`] before loading any data.
pub struct DataLoader {
    backend: Backend,
}

/// Check that the caller is allowed to access the target user's data.
fn check_user_access(security_ctx: &SecurityContext, user_id: &UserId) -> CalceResult<()> {
    if !permissions::can_access_user_data(security_ctx, user_id) {
        return Err(CalceError::Unauthorized {
            requester: security_ctx.user_id.clone(),
            target: user_id.clone(),
        });
    }
    Ok(())
}

impl DataLoader {
    pub fn new(market_data_repo: MarketDataRepo, user_data_repo: UserDataRepo) -> Self {
        Self {
            backend: Backend::Postgres {
                market_data_repo,
                user_data_repo,
            },
        }
    }

    pub fn in_memory(
        market_data: InMemoryMarketDataService,
        user_data: InMemoryUserDataService,
    ) -> Self {
        Self {
            backend: Backend::InMemory {
                market_data: Box::new(market_data),
                user_data,
            },
        }
    }

    /// # Errors
    ///
    /// Returns `Unauthorized` if the security context lacks access.
    /// Returns `NoTradesFound` if the user has no trades.
    /// Propagates database errors.
    pub async fn load_trades(
        &self,
        security_ctx: &SecurityContext,
        user_id: &UserId,
    ) -> CalceResult<Vec<Trade>> {
        check_user_access(security_ctx, user_id)?;

        match &self.backend {
            Backend::InMemory { user_data, .. } => user_data
                .trades_for(user_id)
                .ok_or_else(|| CalceError::NoTradesFound(user_id.clone())),
            Backend::Postgres { user_data_repo, .. } => {
                let trades = user_data_repo
                    .get_trades(user_id)
                    .await
                    .map_err(CalceError::from)?;
                if trades.is_empty() {
                    return Err(CalceError::NoTradesFound(user_id.clone()));
                }
                Ok(trades)
            }
        }
    }

    /// Load trades + all market data needed for user-scoped calculations.
    ///
    /// # Errors
    ///
    /// Returns `Unauthorized` if the security context lacks access.
    /// Returns `NoTradesFound` if the user has no trades.
    /// Propagates price/FX/database errors.
    pub async fn load_user_portfolio(
        &self,
        security_ctx: &SecurityContext,
        user_id: &UserId,
        base_currency: Currency,
        date_range: &DateRange,
    ) -> CalceResult<PortfolioData> {
        check_user_access(security_ctx, user_id)?;

        match &self.backend {
            Backend::InMemory {
                market_data,
                user_data,
            } => {
                let trades = user_data
                    .trades_for(user_id)
                    .ok_or_else(|| CalceError::NoTradesFound(user_id.clone()))?;
                Ok(PortfolioData {
                    trades,
                    market_data: *market_data.clone(),
                })
            }
            Backend::Postgres {
                market_data_repo,
                user_data_repo,
            } => {
                let trades = user_data_repo
                    .get_trades(user_id)
                    .await
                    .map_err(CalceError::from)?;
                if trades.is_empty() {
                    return Err(CalceError::NoTradesFound(user_id.clone()));
                }

                let instruments = unique_instruments(&trades);
                let currencies = unique_currencies(&trades);

                let market_data = load_market_data(
                    market_data_repo,
                    &instruments,
                    &currencies,
                    base_currency,
                    date_range,
                )
                .await?;

                Ok(PortfolioData {
                    trades,
                    market_data,
                })
            }
        }
    }

    /// # Errors
    ///
    /// Propagates database errors.
    pub async fn list_users(&self) -> CalceResult<Vec<UserSummary>> {
        match &self.backend {
            Backend::InMemory { .. } => Ok(vec![]),
            Backend::Postgres { user_data_repo, .. } => {
                let rows = user_data_repo
                    .list_users()
                    .await
                    .map_err(CalceError::from)?;
                Ok(rows
                    .into_iter()
                    .map(|(id, email, trade_count)| UserSummary {
                        id,
                        email,
                        trade_count,
                    })
                    .collect())
            }
        }
    }

    /// # Errors
    ///
    /// Propagates database errors.
    pub async fn list_instruments(&self) -> CalceResult<Vec<InstrumentSummary>> {
        match &self.backend {
            Backend::InMemory { market_data, .. } => Ok(market_data
                .instrument_ids()
                .into_iter()
                .map(|id| InstrumentSummary {
                    id: id.as_str().to_owned(),
                    currency: String::new(),
                    name: None,
                })
                .collect()),
            Backend::Postgres {
                market_data_repo, ..
            } => {
                let rows = market_data_repo
                    .list_instruments()
                    .await
                    .map_err(CalceError::from)?;
                Ok(rows
                    .into_iter()
                    .map(|(id, currency, name)| InstrumentSummary { id, currency, name })
                    .collect())
            }
        }
    }

    /// # Errors
    ///
    /// Propagates database errors.
    pub async fn data_stats(&self) -> CalceResult<DataStats> {
        match &self.backend {
            Backend::InMemory {
                market_data,
                user_data,
            } => Ok(DataStats {
                user_count: i64::try_from(user_data.user_count()).unwrap_or(0),
                instrument_count: i64::try_from(market_data.instrument_count()).unwrap_or(0),
                trade_count: 0,
                price_count: i64::try_from(market_data.price_count()).unwrap_or(0),
                fx_rate_count: i64::try_from(market_data.fx_rate_count()).unwrap_or(0),
            }),
            Backend::Postgres {
                market_data_repo,
                user_data_repo,
            } => {
                let (user_count, trade_count) = user_data_repo
                    .count_users_and_trades()
                    .await
                    .map_err(CalceError::from)?;
                let (instrument_count, price_count, fx_rate_count) = market_data_repo
                    .count_market_data()
                    .await
                    .map_err(CalceError::from)?;
                Ok(DataStats {
                    user_count,
                    instrument_count,
                    trade_count,
                    price_count,
                    fx_rate_count,
                })
            }
        }
    }

    /// Price history for a single instrument.
    ///
    /// # Errors
    ///
    /// Returns `PriceNotFound` if no prices exist in the range.
    /// Propagates database errors.
    pub async fn price_history(
        &self,
        instrument: &InstrumentId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> CalceResult<Vec<(NaiveDate, f64)>> {
        match &self.backend {
            Backend::InMemory { market_data, .. } => {
                let history: Vec<(NaiveDate, Price)> =
                    market_data.get_price_history(instrument, from, to)?;
                Ok(history.into_iter().map(|(d, p)| (d, p.value())).collect())
            }
            Backend::Postgres {
                market_data_repo, ..
            } => {
                let history = market_data_repo
                    .get_price_history(instrument, from, to)
                    .await
                    .map_err(CalceError::from)?;
                Ok(history.into_iter().map(|(d, p)| (d, p.value())).collect())
            }
        }
    }

    /// Load market data for specific instruments (no user data needed).
    ///
    /// This is not user-scoped, so no access check is required.
    ///
    /// # Errors
    ///
    /// Propagates price/FX/database errors.
    pub async fn load_instrument_data(
        &self,
        instruments: &[InstrumentId],
        date_range: &DateRange,
    ) -> CalceResult<InMemoryMarketDataService> {
        match &self.backend {
            Backend::InMemory { market_data, .. } => Ok(*market_data.clone()),
            Backend::Postgres {
                market_data_repo, ..
            } => {
                let mut svc = InMemoryMarketDataService::new();
                for instrument in instruments {
                    let history = market_data_repo
                        .get_price_history(instrument, date_range.from, date_range.to)
                        .await
                        .map_err(CalceError::from)?;
                    for (date, price) in history {
                        svc.add_price(instrument, date, price);
                    }
                }
                svc.freeze();
                Ok(svc)
            }
        }
    }

    /// Run a calculation against the market data without cloning it.
    ///
    /// For the in-memory backend this borrows directly; for Postgres it loads
    /// the requested instruments into a temporary service.
    ///
    /// # Errors
    ///
    /// Propagates price/FX/database errors.
    pub async fn with_market_data<T>(
        &self,
        instruments: &[InstrumentId],
        date_range: &DateRange,
        f: impl FnOnce(&dyn MarketDataService) -> CalceResult<T>,
    ) -> CalceResult<T> {
        match &self.backend {
            Backend::InMemory { market_data, .. } => f(market_data.as_ref()),
            Backend::Postgres { .. } => {
                let svc = self.load_instrument_data(instruments, date_range).await?;
                f(&svc)
            }
        }
    }
}

#[derive(Serialize)]
pub struct UserSummary {
    pub id: String,
    pub email: Option<String>,
    pub trade_count: i64,
}

#[derive(Serialize)]
pub struct InstrumentSummary {
    pub id: String,
    pub currency: String,
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct DataStats {
    pub user_count: i64,
    pub instrument_count: i64,
    pub trade_count: i64,
    pub price_count: i64,
    pub fx_rate_count: i64,
}

fn unique_instruments(trades: &[Trade]) -> Vec<InstrumentId> {
    let mut instruments: Vec<InstrumentId> =
        trades.iter().map(|t| t.instrument_id.clone()).collect();
    instruments.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    instruments.dedup_by(|a, b| a.as_str() == b.as_str());
    instruments
}

fn unique_currencies(trades: &[Trade]) -> Vec<Currency> {
    let mut currencies: Vec<Currency> = trades.iter().map(|t| t.currency).collect();
    currencies.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    currencies.dedup();
    currencies
}

async fn load_market_data(
    repo: &MarketDataRepo,
    instruments: &[InstrumentId],
    currencies: &[Currency],
    base_currency: Currency,
    date_range: &DateRange,
) -> CalceResult<InMemoryMarketDataService> {
    let mut svc = InMemoryMarketDataService::new();

    for instrument in instruments {
        let history = repo
            .get_price_history(instrument, date_range.from, date_range.to)
            .await
            .map_err(CalceError::from)?;
        for (date, price) in history {
            svc.add_price(instrument, date, price);
        }
    }

    let fx_pairs: Vec<(Currency, Currency)> = currencies
        .iter()
        .filter(|c| **c != base_currency)
        .map(|c| (*c, base_currency))
        .collect();

    for &(from_ccy, to_ccy) in &fx_pairs {
        let rates = repo
            .get_fx_rate_history(from_ccy, to_ccy, date_range.from, date_range.to)
            .await
            .map_err(CalceError::from)?;
        for (date, rate) in rates {
            svc.add_fx_rate(rate, date);
        }
    }

    svc.freeze();
    Ok(svc)
}
