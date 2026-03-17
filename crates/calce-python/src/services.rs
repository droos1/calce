use chrono::NaiveDate;
use pyo3::prelude::*;

use calce_core::domain::fx_rate::FxRate;
use calce_core::domain::instrument::InstrumentId;
use calce_core::domain::price::Price;
use calce_data::InMemoryMarketDataService;
use calce_data::user_data_store::UserDataStore;

use crate::domain::{Currency, Trade};

#[pyclass]
pub struct MarketData {
    pub inner: InMemoryMarketDataService,
}

#[pymethods]
impl MarketData {
    #[new]
    fn new() -> Self {
        MarketData {
            inner: InMemoryMarketDataService::new(),
        }
    }

    fn add_price(&mut self, instrument_id: &str, date: NaiveDate, price: f64) {
        self.inner
            .add_price(&InstrumentId::new(instrument_id), date, Price::new(price));
    }

    fn add_fx_rate(
        &mut self,
        from_currency: &Currency,
        to_currency: &Currency,
        rate: f64,
        date: NaiveDate,
    ) {
        self.inner.add_fx_rate(
            FxRate::new(from_currency.inner, to_currency.inner, rate),
            date,
        );
    }

    fn freeze(&mut self) {
        self.inner.freeze();
    }
}

#[pyclass]
pub struct UserData {
    pub inner: UserDataStore,
}

#[pymethods]
impl UserData {
    #[new]
    fn new() -> Self {
        UserData {
            inner: UserDataStore::new(),
        }
    }

    fn add_trade(&mut self, trade: &Trade) {
        self.inner.add_trade(trade.inner.clone());
    }
}
