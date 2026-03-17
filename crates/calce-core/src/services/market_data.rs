use chrono::NaiveDate;

use crate::domain::currency::Currency;
use crate::domain::fx_rate::FxRate;
use crate::domain::instrument::InstrumentId;
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
}
