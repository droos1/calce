use chrono::NaiveDate;

use super::currency::Currency;
use super::instrument::InstrumentId;
use super::price::Price;
use super::quantity::Quantity;
use super::user::UserId;

/// Quantity is signed: positive = buy, negative = sell.
#[derive(Clone, Debug)]
pub struct Trade {
    pub user_id: UserId,
    pub instrument_id: InstrumentId,
    pub quantity: Quantity,
    pub price: Price,
    pub currency: Currency,
    pub date: NaiveDate,
}
