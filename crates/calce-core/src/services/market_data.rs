use chrono::NaiveDate;

use crate::domain::currency::Currency;
use crate::domain::fx_rate::FxRate;
use crate::domain::instrument::{InstrumentId, InstrumentType};
use crate::domain::price::Price;
use crate::error::CalceResult;

pub trait MarketDataService: Send + Sync {
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
    fn get_fx_rate(&self, from: Currency, to: Currency, date: NaiveDate) -> CalceResult<FxRate>;

    /// Returns the instrument type classification. Defaults to `Other`.
    fn get_instrument_type(&self, _instrument: &InstrumentId) -> InstrumentType {
        InstrumentType::Other
    }

    /// Returns allocation weights for an instrument in a given dimension.
    ///
    /// For example, dimension "sector" might return
    /// `[("Information Technology", 0.30), ("Health Care", 0.13)]` for an ETF,
    /// or `[("Information Technology", 1.0)]` for a single-sector stock.
    ///
    /// Returns an empty vec if no allocation data is available (the caller
    /// should treat this as "Uncategorized" at weight 1.0).
    fn get_allocations(&self, _instrument: &InstrumentId, _dimension: &str) -> Vec<(String, f64)> {
        Vec::new()
    }
}
