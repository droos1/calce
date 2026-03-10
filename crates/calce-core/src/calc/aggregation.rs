use std::collections::HashMap;

use chrono::NaiveDate;

use crate::domain::currency::Currency;
use crate::domain::instrument::InstrumentId;
use crate::domain::position::Position;
use crate::domain::quantity::Quantity;
use crate::domain::trade::Trade;
use crate::error::{CalceError, CalceResult};

/// `#CALC_POS_AGG`
///
/// Trades after `as_of_date` are excluded. Fully closed positions (zero net quantity) are omitted.
///
/// # Errors
///
/// Returns `CurrencyConflict` if the same instrument appears with different currencies.
pub fn aggregate_positions(trades: &[Trade], as_of_date: NaiveDate) -> CalceResult<Vec<Position>> {
    let mut net: HashMap<InstrumentId, (Quantity, Currency)> = HashMap::new();
    let mut conflict: Option<(InstrumentId, Currency, Currency)> = None;

    for trade in trades {
        if trade.date <= as_of_date {
            net.entry(trade.instrument_id.clone())
                .and_modify(|(qty, existing_ccy)| {
                    if *existing_ccy == trade.currency {
                        *qty = *qty + trade.quantity;
                    } else if conflict.is_none() {
                        conflict =
                            Some((trade.instrument_id.clone(), *existing_ccy, trade.currency));
                    }
                })
                .or_insert((trade.quantity, trade.currency));
        }
    }

    if let Some((instrument, expected, actual)) = conflict {
        return Err(CalceError::CurrencyConflict {
            instrument,
            expected,
            actual,
        });
    }

    let mut positions: Vec<Position> = net
        .into_iter()
        .filter(|(_, (qty, _))| !qty.is_zero())
        .map(|(id, (qty, ccy))| Position {
            instrument_id: id,
            quantity: qty,
            currency: ccy,
        })
        .collect();

    // Sort for deterministic output
    positions.sort_by(|a, b| a.instrument_id.as_str().cmp(b.instrument_id.as_str()));
    Ok(positions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::currency::Currency;
    use crate::domain::price::Price;
    use crate::domain::user::UserId;

    #[test]
    fn aggregates_buys_and_sells() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).expect("valid test date");
        let usd = Currency::new("USD");
        let aapl = InstrumentId::new("AAPL");
        let alice = UserId::new("alice");
        let acct = AccountId::new("alice-usd");

        let trades = vec![
            Trade {
                user_id: alice.clone(),
                account_id: acct.clone(),
                instrument_id: aapl.clone(),
                quantity: Quantity::new(100.0),
                price: Price::new(145.0),
                currency: usd,
                date,
            },
            Trade {
                user_id: alice,
                account_id: acct,
                instrument_id: aapl,
                quantity: Quantity::new(-30.0),
                price: Price::new(150.0),
                currency: usd,
                date,
            },
        ];

        let positions = aggregate_positions(&trades, date).unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].quantity.value(), 70.0);
    }

    #[test]
    fn fully_closed_position_excluded() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).expect("valid test date");
        let usd = Currency::new("USD");
        let aapl = InstrumentId::new("AAPL");
        let alice = UserId::new("alice");
        let acct = AccountId::new("alice-usd");

        let trades = vec![
            Trade {
                user_id: alice.clone(),
                account_id: acct.clone(),
                instrument_id: aapl.clone(),
                quantity: Quantity::new(100.0),
                price: Price::new(145.0),
                currency: usd,
                date,
            },
            Trade {
                user_id: alice,
                account_id: acct,
                instrument_id: aapl,
                quantity: Quantity::new(-100.0),
                price: Price::new(150.0),
                currency: usd,
                date,
            },
        ];

        let positions = aggregate_positions(&trades, date).unwrap();
        assert!(positions.is_empty());
    }

    #[test]
    fn filters_by_as_of_date() {
        let early = NaiveDate::from_ymd_opt(2025, 1, 10).expect("valid test date");
        let late = NaiveDate::from_ymd_opt(2025, 1, 20).expect("valid test date");
        let usd = Currency::new("USD");
        let aapl = InstrumentId::new("AAPL");
        let alice = UserId::new("alice");
        let acct = AccountId::new("alice-usd");

        let trades = vec![
            Trade {
                user_id: alice.clone(),
                account_id: acct.clone(),
                instrument_id: aapl.clone(),
                quantity: Quantity::new(50.0),
                price: Price::new(140.0),
                currency: usd,
                date: early,
            },
            Trade {
                user_id: alice,
                account_id: acct,
                instrument_id: aapl,
                quantity: Quantity::new(30.0),
                price: Price::new(150.0),
                currency: usd,
                date: late,
            },
        ];

        let positions = aggregate_positions(&trades, early).unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].quantity.value(), 50.0);
    }

    #[test]
    fn mixed_currencies_for_same_instrument_rejected() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).expect("valid test date");
        let usd = Currency::new("USD");
        let eur = Currency::new("EUR");
        let aapl = InstrumentId::new("AAPL");
        let alice = UserId::new("alice");
        let acct_usd = AccountId::new("alice-usd");
        let acct_eur = AccountId::new("alice-eur");

        let trades = vec![
            Trade {
                user_id: alice.clone(),
                account_id: acct_usd,
                instrument_id: aapl.clone(),
                quantity: Quantity::new(100.0),
                price: Price::new(145.0),
                currency: usd,
                date,
            },
            Trade {
                user_id: alice,
                account_id: acct_eur,
                instrument_id: aapl,
                quantity: Quantity::new(50.0),
                price: Price::new(145.0),
                currency: eur,
                date,
            },
        ];

        let result = aggregate_positions(&trades, date);
        assert!(matches!(
            result.unwrap_err(),
            CalceError::CurrencyConflict { .. }
        ));
    }
}
