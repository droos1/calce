use super::currency::Currency;
use super::instrument::InstrumentId;
use super::quantity::Quantity;

/// Net holding — no market values attached (see `ValuedPosition` for that).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
    pub instrument_id: InstrumentId,
    pub quantity: Quantity,
    pub currency: Currency,
}
