use std::collections::HashMap;

use crate::domain::instrument::InstrumentType;
use crate::domain::money::Money;
use crate::services::market_data::MarketDataService;

use super::market_value::ValuedPosition;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeAllocationEntry {
    pub instrument_type: InstrumentType,
    pub market_value: Money,
    pub weight: f64,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeAllocation {
    pub entries: Vec<TypeAllocationEntry>,
    pub total: Money,
}

/// `#CALC_ALLOC`
///
/// Groups valued positions by instrument type and computes each type's share
/// of total portfolio value. Types are resolved via `market_data`, keeping
/// classification separate from valuation.
///
/// Entries are sorted by descending weight.
pub fn type_allocation(
    positions: &[ValuedPosition],
    total: Money,
    market_data: &dyn MarketDataService,
) -> TypeAllocation {
    let mut by_type: HashMap<InstrumentType, f64> = HashMap::new();

    for pos in positions {
        let itype = market_data.get_instrument_type(&pos.instrument_id);
        *by_type.entry(itype).or_default() += pos.market_value_base.amount;
    }

    let total_amount = total.amount;
    let mut entries: Vec<TypeAllocationEntry> = by_type
        .into_iter()
        .map(|(instrument_type, amount)| {
            let weight = if total_amount == 0.0 {
                0.0
            } else {
                amount / total_amount
            };
            TypeAllocationEntry {
                instrument_type,
                market_value: Money::new(amount, total.currency),
                weight,
            }
        })
        .collect();

    entries.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    TypeAllocation { entries, total }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::currency::Currency;
    use crate::domain::instrument::InstrumentId;
    use crate::domain::price::Price;
    use crate::domain::quantity::Quantity;
    use crate::services::test_market_data::TestMarketData;

    #[test]
    fn single_type_gets_full_weight() {
        let sek = Currency::new("SEK");
        let aapl = InstrumentId::new("AAPL");

        let mut md = TestMarketData::new();
        md.add_instrument_type(&aapl, InstrumentType::Stock);

        let positions = vec![ValuedPosition {
            instrument_id: aapl,
            quantity: Quantity::new(100.0),
            currency: sek,
            price: Price::new(150.0),
            market_value: Money::new(15_000.0, sek),
            market_value_base: Money::new(15_000.0, sek),
        }];
        let total = Money::new(15_000.0, sek);

        let alloc = type_allocation(&positions, total, &md);
        assert_eq!(alloc.entries.len(), 1);
        assert_eq!(alloc.entries[0].instrument_type, InstrumentType::Stock);
        assert!((alloc.entries[0].weight - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn multiple_types_sorted_by_weight() {
        let sek = Currency::new("SEK");
        let aapl = InstrumentId::new("AAPL");
        let spy = InstrumentId::new("SPY");
        let bond = InstrumentId::new("BOND1");

        let mut md = TestMarketData::new();
        md.add_instrument_type(&aapl, InstrumentType::Stock);
        md.add_instrument_type(&spy, InstrumentType::Etf);
        md.add_instrument_type(&bond, InstrumentType::Bond);

        let positions = vec![
            ValuedPosition {
                instrument_id: aapl,
                quantity: Quantity::new(10.0),
                currency: sek,
                price: Price::new(100.0),
                market_value: Money::new(1_000.0, sek),
                market_value_base: Money::new(1_000.0, sek),
            },
            ValuedPosition {
                instrument_id: spy,
                quantity: Quantity::new(10.0),
                currency: sek,
                price: Price::new(500.0),
                market_value: Money::new(5_000.0, sek),
                market_value_base: Money::new(5_000.0, sek),
            },
            ValuedPosition {
                instrument_id: bond,
                quantity: Quantity::new(10.0),
                currency: sek,
                price: Price::new(400.0),
                market_value: Money::new(4_000.0, sek),
                market_value_base: Money::new(4_000.0, sek),
            },
        ];
        let total = Money::new(10_000.0, sek);

        let alloc = type_allocation(&positions, total, &md);
        assert_eq!(alloc.entries.len(), 3);
        // Sorted descending: ETF (50%), Bond (40%), Stock (10%)
        assert_eq!(alloc.entries[0].instrument_type, InstrumentType::Etf);
        assert!((alloc.entries[0].weight - 0.5).abs() < f64::EPSILON);
        assert_eq!(alloc.entries[1].instrument_type, InstrumentType::Bond);
        assert!((alloc.entries[1].weight - 0.4).abs() < f64::EPSILON);
        assert_eq!(alloc.entries[2].instrument_type, InstrumentType::Stock);
        assert!((alloc.entries[2].weight - 0.1).abs() < f64::EPSILON);

        // Weights sum to ~1.0
        let sum: f64 = alloc.entries.iter().map(|e| e.weight).sum();
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn unknown_instruments_default_to_other() {
        let sek = Currency::new("SEK");
        let mystery = InstrumentId::new("MYSTERY");
        let md = TestMarketData::new();

        let positions = vec![ValuedPosition {
            instrument_id: mystery,
            quantity: Quantity::new(10.0),
            currency: sek,
            price: Price::new(100.0),
            market_value: Money::new(1_000.0, sek),
            market_value_base: Money::new(1_000.0, sek),
        }];
        let total = Money::new(1_000.0, sek);

        let alloc = type_allocation(&positions, total, &md);
        assert_eq!(alloc.entries.len(), 1);
        assert_eq!(alloc.entries[0].instrument_type, InstrumentType::Other);
    }

    #[test]
    fn zero_total_gives_zero_weights() {
        let sek = Currency::new("SEK");
        let total = Money::new(0.0, sek);
        let md = TestMarketData::new();

        let alloc = type_allocation(&[], total, &md);
        assert!(alloc.entries.is_empty());
    }

    /// Positions in different local currencies are aggregated correctly
    /// because allocation uses `market_value_base` (already FX-converted).
    #[test]
    fn mixed_currencies_uses_base_values() {
        let usd = Currency::new("USD");
        let eur = Currency::new("EUR");
        let sek = Currency::new("SEK");
        let aapl = InstrumentId::new("AAPL");
        let vow3 = InstrumentId::new("VOW3");

        let mut md = TestMarketData::new();
        md.add_instrument_type(&aapl, InstrumentType::Stock);
        md.add_instrument_type(&vow3, InstrumentType::Stock);

        // AAPL: local 12,000 USD, base 126,000 SEK
        // VOW3: local 6,000 EUR, base 68,400 SEK
        let positions = vec![
            ValuedPosition {
                instrument_id: aapl,
                quantity: Quantity::new(80.0),
                currency: usd,
                price: Price::new(150.0),
                market_value: Money::new(12_000.0, usd),
                market_value_base: Money::new(126_000.0, sek),
            },
            ValuedPosition {
                instrument_id: vow3,
                quantity: Quantity::new(50.0),
                currency: eur,
                price: Price::new(120.0),
                market_value: Money::new(6_000.0, eur),
                market_value_base: Money::new(68_400.0, sek),
            },
        ];
        let total = Money::new(194_400.0, sek);

        let alloc = type_allocation(&positions, total, &md);

        // Both are Stock → single entry summing the base values
        assert_eq!(alloc.entries.len(), 1);
        assert_eq!(alloc.entries[0].instrument_type, InstrumentType::Stock);
        assert_eq!(alloc.entries[0].market_value.amount, 194_400.0);
        assert_eq!(alloc.entries[0].market_value.currency, sek);
        assert!((alloc.entries[0].weight - 1.0).abs() < f64::EPSILON);
    }
}
