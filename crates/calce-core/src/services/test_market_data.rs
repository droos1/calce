use std::collections::HashMap;

use chrono::NaiveDate;

use crate::domain::currency::Currency;
use crate::domain::fx_rate::FxRate;
use crate::domain::instrument::{InstrumentId, InstrumentType};
use crate::domain::price::Price;
use crate::error::{CalceError, CalceResult};

use super::market_data::MarketDataService;

/// Simple HashMap-based market data for tests.
///
/// Simple HashMap-based market data — no freeze step needed, data is
/// queryable immediately after insertion.
#[derive(Default)]
pub struct TestMarketData {
    prices: HashMap<(InstrumentId, NaiveDate), f64>,
    fx_rates: HashMap<(Currency, Currency, NaiveDate), f64>,
    instrument_types: HashMap<InstrumentId, InstrumentType>,
}

impl TestMarketData {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_price(&mut self, instrument: &InstrumentId, date: NaiveDate, price: Price) {
        self.prices
            .insert((instrument.clone(), date), price.value());
    }

    pub fn add_fx_rate(&mut self, rate: FxRate, date: NaiveDate) {
        self.fx_rates.insert((rate.from, rate.to, date), rate.rate);
    }

    pub fn add_instrument_type(
        &mut self,
        instrument: &InstrumentId,
        instrument_type: InstrumentType,
    ) {
        self.instrument_types
            .insert(instrument.clone(), instrument_type);
    }
}

impl MarketDataService for TestMarketData {
    fn get_price(&self, instrument: &InstrumentId, date: NaiveDate) -> CalceResult<Price> {
        self.prices
            .get(&(instrument.clone(), date))
            .map(|&v| Price::new(v))
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
            .map(|((_, d), &v)| (*d, Price::new(v)))
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

    fn get_fx_rate(&self, from: Currency, to: Currency, date: NaiveDate) -> CalceResult<FxRate> {
        if from == to {
            return Ok(FxRate::identity(from));
        }
        self.fx_rates
            .get(&(from, to, date))
            .map(|&rate| FxRate::new(from, to, rate))
            .ok_or(CalceError::FxRateNotFound { from, to, date })
    }

    fn get_instrument_type(&self, instrument: &InstrumentId) -> InstrumentType {
        self.instrument_types
            .get(instrument)
            .copied()
            .unwrap_or_default()
    }
}
