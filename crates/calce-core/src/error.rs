use chrono::NaiveDate;

use crate::domain::currency::Currency;
use crate::domain::instrument::InstrumentId;
use crate::domain::money::CurrencyMismatch;

pub type CalceResult<T> = Result<T, CalceError>;

#[derive(Debug, thiserror::Error)]
pub enum CalceError {
    #[error("Price not found for {instrument} on {date}")]
    PriceNotFound {
        instrument: InstrumentId,
        date: NaiveDate,
    },

    #[error("FX rate not found for {from}/{to} on {date}")]
    FxRateNotFound {
        from: Currency,
        to: Currency,
        date: NaiveDate,
    },

    #[error("Insufficient data for {instrument}: {reason}")]
    InsufficientData {
        instrument: InstrumentId,
        reason: String,
    },

    #[error("Currency mismatch: {0}")]
    CurrencyMismatch(#[from] CurrencyMismatch),

    #[error("Currency conflict for {instrument}: expected {expected}, got {actual}")]
    CurrencyConflict {
        instrument: InstrumentId,
        expected: Currency,
        actual: Currency,
    },
}
