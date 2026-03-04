use std::collections::HashMap;

use chrono::NaiveDate;

use crate::domain::currency::Currency;
use crate::domain::fx_rate::FxRate;
use crate::domain::instrument::InstrumentId;
use crate::domain::price::Price;
use crate::error::{CalceError, CalceResult};

pub trait MarketDataService {
    /// # Errors
    ///
    /// Returns `PriceNotFound` if no price is available.
    fn get_price(&self, instrument: &InstrumentId, date: NaiveDate) -> CalceResult<Price>;

    /// Returns all available prices for an instrument in `[from, to]`, sorted by date.
    ///
    /// # Errors
    ///
    /// Returns `PriceNotFound` if no prices exist in the range.
    fn get_price_history(
        &self,
        instrument: &InstrumentId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> CalceResult<Vec<(NaiveDate, Price)>>;

    /// # Errors
    ///
    /// Returns `FxRateNotFound` if no rate is available.
    fn get_fx_rate(
        &self,
        from: Currency,
        to: Currency,
        date: NaiveDate,
    ) -> CalceResult<FxRate>;
}

/// In-memory implementation for testing.
#[derive(Default)]
pub struct InMemoryMarketDataService {
    prices: HashMap<(InstrumentId, NaiveDate), Price>,
    fx_rates: HashMap<(Currency, Currency, NaiveDate), FxRate>,
}

impl InMemoryMarketDataService {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_price(&mut self, instrument: &InstrumentId, date: NaiveDate, price: Price) {
        self.prices.insert((instrument.clone(), date), price);
    }

    pub fn add_fx_rate(&mut self, rate: FxRate, date: NaiveDate) {
        self.fx_rates.insert((rate.from, rate.to, date), rate);
    }
}

impl MarketDataService for InMemoryMarketDataService {
    fn get_price(&self, instrument: &InstrumentId, date: NaiveDate) -> CalceResult<Price> {
        self.prices
            .get(&(instrument.clone(), date))
            .copied()
            .ok_or_else(|| CalceError::PriceNotFound {
                instrument: instrument.clone(),
                date,
            })
    }

    fn get_price_history(
        &self,
        instrument: &InstrumentId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> CalceResult<Vec<(NaiveDate, Price)>> {
        let mut result: Vec<(NaiveDate, Price)> = self
            .prices
            .iter()
            .filter(|((id, d), _)| id == instrument && *d >= from && *d <= to)
            .map(|((_, d), p)| (*d, *p))
            .collect();
        result.sort_by_key(|(d, _)| *d);
        if result.is_empty() {
            return Err(CalceError::PriceNotFound {
                instrument: instrument.clone(),
                date: from,
            });
        }
        Ok(result)
    }

    fn get_fx_rate(
        &self,
        from: Currency,
        to: Currency,
        date: NaiveDate,
    ) -> CalceResult<FxRate> {
        if from == to {
            return Ok(FxRate::identity(from));
        }
        self.fx_rates
            .get(&(from, to, date))
            .copied()
            .ok_or(CalceError::FxRateNotFound { from, to, date })
    }
}
