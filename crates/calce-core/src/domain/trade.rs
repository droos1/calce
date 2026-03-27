use chrono::NaiveDate;

use super::account::AccountId;
use super::currency::Currency;
use super::instrument::InstrumentId;
use super::price::Price;
use super::quantity::Quantity;
use super::user::UserId;

int_id!(TradeId);

/// Quantity is signed: positive = buy, negative = sell.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trade {
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub id: Option<TradeId>,
    pub user_id: UserId,
    pub account_id: AccountId,
    pub instrument_id: InstrumentId,
    pub quantity: Quantity,
    pub price: Price,
    pub currency: Currency,
    pub date: NaiveDate,
}
