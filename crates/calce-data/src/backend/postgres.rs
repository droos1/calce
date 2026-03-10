use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDate;

use calce_core::domain::currency::Currency;
use calce_core::domain::instrument::InstrumentId;
use calce_core::domain::price::Price;
use calce_core::domain::trade::Trade;
use calce_core::domain::user::UserId;
use calce_core::services::market_data::InMemoryMarketDataService;

use crate::error::{DataError, DataResult};

use super::{DataBackend, DataStats, DateRange, InstrumentSummary, UserSummary};
use crate::repo::market_data::MarketDataRepo;
use crate::repo::user_data::UserDataRepo;

pub struct PostgresBackend {
    market_data_repo: MarketDataRepo,
    user_data_repo: UserDataRepo,
}

impl PostgresBackend {
    pub fn new(market_data_repo: MarketDataRepo, user_data_repo: UserDataRepo) -> Self {
        Self {
            market_data_repo,
            user_data_repo,
        }
    }
}

#[async_trait]
impl DataBackend for PostgresBackend {
    async fn load_trades(&self, user_id: &UserId) -> DataResult<Vec<Trade>> {
        let trades = self.user_data_repo.get_trades(user_id).await?;
        if trades.is_empty() {
            return Err(DataError::NoTradesFound(user_id.clone()));
        }
        Ok(trades)
    }

    async fn load_market_data(
        &self,
        instruments: &[InstrumentId],
        currencies: &[Currency],
        base_currency: Currency,
        date_range: &DateRange,
    ) -> DataResult<Arc<InMemoryMarketDataService>> {
        let mut svc = InMemoryMarketDataService::new();

        let price_map = self
            .market_data_repo
            .get_price_history_batch(instruments, date_range.from, date_range.to)
            .await?;
        for (instrument, history) in price_map {
            for (date, price) in history {
                svc.add_price(&instrument, date, price);
            }
        }

        let fx_pairs: Vec<(Currency, Currency)> = currencies
            .iter()
            .filter(|c| **c != base_currency)
            .map(|c| (*c, base_currency))
            .collect();

        let fx_history = self
            .market_data_repo
            .get_fx_rate_history_batch(&fx_pairs, date_range.from, date_range.to)
            .await?;
        for (date, rate) in fx_history {
            svc.add_fx_rate(rate, date);
        }

        svc.freeze();
        Ok(Arc::new(svc))
    }

    async fn list_users(&self) -> DataResult<Vec<UserSummary>> {
        let rows = self.user_data_repo.list_users().await?;
        Ok(rows
            .into_iter()
            .map(|(id, email, trade_count)| UserSummary {
                id,
                email,
                trade_count,
            })
            .collect())
    }

    async fn list_instruments(&self) -> DataResult<Vec<InstrumentSummary>> {
        let rows = self.market_data_repo.list_instruments().await?;
        Ok(rows
            .into_iter()
            .map(|(id, currency, name)| InstrumentSummary { id, currency, name })
            .collect())
    }

    async fn data_stats(&self) -> DataResult<DataStats> {
        let (user_trade, market) = tokio::try_join!(
            self.user_data_repo.count_users_and_trades(),
            self.market_data_repo.count_market_data(),
        )?;
        let (user_count, trade_count) = user_trade;
        let (instrument_count, price_count, fx_rate_count) = market;
        Ok(DataStats {
            user_count,
            instrument_count,
            trade_count,
            price_count,
            fx_rate_count,
        })
    }

    async fn price_history(
        &self,
        instrument: &InstrumentId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> DataResult<Vec<(NaiveDate, f64)>> {
        let history: Vec<(NaiveDate, Price)> = self
            .market_data_repo
            .get_price_history(instrument, from, to)
            .await?;
        Ok(history.into_iter().map(|(d, p)| (d, p.value())).collect())
    }
}
