use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};

use calce_core::domain::currency::Currency;
use calce_core::domain::fx_rate::FxRate;
use calce_core::domain::instrument::{InstrumentId, InstrumentType};
use calce_core::domain::price::Price;

/// Accumulates market data for bulk-loading into [`ConcurrentMarketData`].
///
/// Call [`add_price`], [`add_fx_rate`], [`add_instrument_type`], and
/// [`add_allocation`] to build up data, then pass the builder to
/// [`ConcurrentMarketData::from_builder`] to materialise the concurrent store.
///
/// [`ConcurrentMarketData`]: crate::concurrent_market_data::ConcurrentMarketData
#[derive(Default)]
pub struct MarketDataBuilder {
    pub(crate) prices: HashMap<InstrumentId, Vec<(i32, f64)>>,
    pub(crate) fx_rates: HashMap<(Currency, Currency), Vec<(i32, f64)>>,
    pub(crate) instrument_types: HashMap<InstrumentId, InstrumentType>,
    pub(crate) allocations: HashMap<InstrumentId, HashMap<String, Vec<(String, f64)>>>,
}

impl MarketDataBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_price(&mut self, instrument: &InstrumentId, date: NaiveDate, price: Price) {
        self.prices
            .entry(instrument.clone())
            .or_default()
            .push((date.num_days_from_ce(), price.value()));
    }

    pub fn add_fx_rate(&mut self, rate: FxRate, date: NaiveDate) {
        self.fx_rates
            .entry((rate.from, rate.to))
            .or_default()
            .push((date.num_days_from_ce(), rate.rate));
    }

    pub fn add_instrument_type(
        &mut self,
        instrument: &InstrumentId,
        instrument_type: InstrumentType,
    ) {
        self.instrument_types
            .insert(instrument.clone(), instrument_type);
    }

    pub fn add_allocation(
        &mut self,
        instrument: &InstrumentId,
        dimension: &str,
        key: &str,
        weight: f64,
    ) {
        self.allocations
            .entry(instrument.clone())
            .or_default()
            .entry(dimension.to_owned())
            .or_default()
            .push((key.to_owned(), weight));
    }
}
