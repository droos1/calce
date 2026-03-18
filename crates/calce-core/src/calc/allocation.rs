use std::collections::HashMap;

use crate::domain::instrument::{InstrumentId, InstrumentType};
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

/// `#CALC_ALLOC_INSTYPE`
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

// ---------------------------------------------------------------------------
// Weighted allocation (generic, multi-dimensional)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AllocationEntry {
    pub key: String,
    pub market_value: Money,
    pub weight: f64,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AllocationResult {
    pub dimension: String,
    pub entries: Vec<AllocationEntry>,
    pub total: Money,
}

/// `#CALC_ALLOC_WEIGHTED`
///
/// Generic weighted allocation engine. Distributes each position's base-currency
/// market value across categories according to per-instrument allocation weights.
///
/// Instruments with no allocation data for the requested dimension are mapped to
/// "Uncategorized" at weight 1.0.
///
/// Entries are sorted by descending portfolio weight.
pub fn weighted_allocation(
    positions: &[ValuedPosition],
    total: Money,
    dimension: &str,
    get_weights: impl Fn(&InstrumentId) -> Vec<(String, f64)>,
) -> AllocationResult {
    let mut by_key: HashMap<String, f64> = HashMap::new();

    for pos in positions {
        let weights = get_weights(&pos.instrument_id);
        if weights.is_empty() {
            // No allocation data → full value to "Uncategorized"
            *by_key.entry("Uncategorized".to_owned()).or_default() += pos.market_value_base.amount;
        } else {
            for (key, w) in weights {
                *by_key.entry(key).or_default() += pos.market_value_base.amount * w;
            }
        }
    }

    let total_amount = total.amount;
    let mut entries: Vec<AllocationEntry> = by_key
        .into_iter()
        .map(|(key, amount)| {
            let weight = if total_amount == 0.0 {
                0.0
            } else {
                amount / total_amount
            };
            AllocationEntry {
                key,
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

    AllocationResult {
        dimension: dimension.to_owned(),
        entries,
        total,
    }
}

/// `#CALC_ALLOC_SECTOR`
///
/// Sector allocation via `#CALC_ALLOC_WEIGHTED` with dimension "sector".
pub fn sector_allocation(
    positions: &[ValuedPosition],
    total: Money,
    market_data: &dyn MarketDataService,
) -> AllocationResult {
    weighted_allocation(positions, total, "sector", |id| {
        market_data.get_allocations(id, "sector")
    })
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

    // -----------------------------------------------------------------------
    // Weighted allocation tests (#CALC_ALLOC_WEIGHTED / #CALC_ALLOC_SECTOR)
    // -----------------------------------------------------------------------

    fn make_position(id: &str, base_value: f64, currency: Currency) -> ValuedPosition {
        ValuedPosition {
            instrument_id: InstrumentId::new(id),
            quantity: Quantity::new(1.0),
            currency,
            price: Price::new(base_value),
            market_value: Money::new(base_value, currency),
            market_value_base: Money::new(base_value, currency),
        }
    }

    /// Stock with a single sector at weight 1.0 → 100% of its value in that sector.
    #[test]
    fn weighted_single_category_stock() {
        let sek = Currency::new("SEK");
        let positions = vec![make_position("AAPL", 100_000.0, sek)];
        let total = Money::new(100_000.0, sek);

        let alloc = weighted_allocation(&positions, total, "sector", |id| {
            if id.as_str() == "AAPL" {
                vec![("Information Technology".to_owned(), 1.0)]
            } else {
                vec![]
            }
        });

        assert_eq!(alloc.dimension, "sector");
        assert_eq!(alloc.entries.len(), 1);
        assert_eq!(alloc.entries[0].key, "Information Technology");
        assert_eq!(alloc.entries[0].market_value.amount, 100_000.0);
        assert!((alloc.entries[0].weight - 1.0).abs() < f64::EPSILON);
    }

    /// ETF with multi-sector weights → value distributed proportionally.
    #[test]
    fn weighted_multi_category_fund() {
        let sek = Currency::new("SEK");
        // SPY worth 100,000 SEK, split 60% tech / 40% health care
        let positions = vec![make_position("SPY", 100_000.0, sek)];
        let total = Money::new(100_000.0, sek);

        let alloc = weighted_allocation(&positions, total, "sector", |_| {
            vec![
                ("Information Technology".to_owned(), 0.6),
                ("Health Care".to_owned(), 0.4),
            ]
        });

        assert_eq!(alloc.entries.len(), 2);
        // Sorted descending: tech (60%) then health care (40%)
        assert_eq!(alloc.entries[0].key, "Information Technology");
        assert!((alloc.entries[0].market_value.amount - 60_000.0).abs() < 1e-10);
        assert!((alloc.entries[0].weight - 0.6).abs() < 1e-10);
        assert_eq!(alloc.entries[1].key, "Health Care");
        assert!((alloc.entries[1].market_value.amount - 40_000.0).abs() < 1e-10);
        assert!((alloc.entries[1].weight - 0.4).abs() < 1e-10);
    }

    /// Stock and fund overlapping in the same sector → values combine.
    #[test]
    fn weighted_mixed_portfolio_overlap() {
        let sek = Currency::new("SEK");
        // AAPL: 60,000 SEK, 100% Information Technology
        // SPY:  40,000 SEK, 50% Information Technology / 50% Health Care
        // Total: 100,000 SEK
        // Info Tech: 60,000 + 20,000 = 80,000 (80%)
        // Health Care: 20,000 (20%)
        let positions = vec![
            make_position("AAPL", 60_000.0, sek),
            make_position("SPY", 40_000.0, sek),
        ];
        let total = Money::new(100_000.0, sek);

        let alloc = weighted_allocation(&positions, total, "sector", |id| match id.as_str() {
            "AAPL" => vec![("Information Technology".to_owned(), 1.0)],
            "SPY" => vec![
                ("Information Technology".to_owned(), 0.5),
                ("Health Care".to_owned(), 0.5),
            ],
            _ => vec![],
        });

        assert_eq!(alloc.entries.len(), 2);
        assert_eq!(alloc.entries[0].key, "Information Technology");
        assert!((alloc.entries[0].market_value.amount - 80_000.0).abs() < 1e-10);
        assert!((alloc.entries[0].weight - 0.8).abs() < 1e-10);
        assert_eq!(alloc.entries[1].key, "Health Care");
        assert!((alloc.entries[1].market_value.amount - 20_000.0).abs() < 1e-10);
        assert!((alloc.entries[1].weight - 0.2).abs() < 1e-10);
    }

    /// Instrument with no allocation data → "Uncategorized" at full weight.
    #[test]
    fn weighted_missing_allocations_uncategorized() {
        let sek = Currency::new("SEK");
        let positions = vec![make_position("UNKNOWN", 50_000.0, sek)];
        let total = Money::new(50_000.0, sek);

        let alloc = weighted_allocation(&positions, total, "sector", |_| vec![]);

        assert_eq!(alloc.entries.len(), 1);
        assert_eq!(alloc.entries[0].key, "Uncategorized");
        assert_eq!(alloc.entries[0].market_value.amount, 50_000.0);
        assert!((alloc.entries[0].weight - 1.0).abs() < f64::EPSILON);
    }

    /// Weights summing to less than 1.0 — unallocated portion is lost.
    #[test]
    fn weighted_partial_weights() {
        let sek = Currency::new("SEK");
        // Fund with 80% allocated (20% is cash/other not attributed)
        let positions = vec![make_position("FUND", 100_000.0, sek)];
        let total = Money::new(100_000.0, sek);

        let alloc = weighted_allocation(&positions, total, "sector", |_| {
            vec![("Equity".to_owned(), 0.5), ("Fixed Income".to_owned(), 0.3)]
        });

        assert_eq!(alloc.entries.len(), 2);
        // Only 80,000 of 100,000 is attributed
        let attributed: f64 = alloc.entries.iter().map(|e| e.market_value.amount).sum();
        assert!((attributed - 80_000.0).abs() < 1e-10);
        // Weights sum to 0.8, not 1.0
        let weight_sum: f64 = alloc.entries.iter().map(|e| e.weight).sum();
        assert!((weight_sum - 0.8).abs() < 1e-10);
    }

    /// Zero total → all weights are zero.
    #[test]
    fn weighted_zero_total() {
        let sek = Currency::new("SEK");
        let total = Money::new(0.0, sek);

        let alloc = weighted_allocation(&[], total, "sector", |_| vec![]);

        assert!(alloc.entries.is_empty());
    }

    /// Sector allocation via MarketDataService (convenience wrapper).
    #[test]
    fn sector_allocation_via_market_data() {
        let sek = Currency::new("SEK");
        let aapl = InstrumentId::new("AAPL");
        let spy = InstrumentId::new("SPY");

        let mut md = TestMarketData::new();
        md.add_allocation(&aapl, "sector", "Information Technology", 1.0);
        md.add_allocation(&spy, "sector", "Information Technology", 0.3);
        md.add_allocation(&spy, "sector", "Health Care", 0.2);
        md.add_allocation(&spy, "sector", "Financials", 0.5);

        let positions = vec![
            make_position("AAPL", 60_000.0, sek),
            make_position("SPY", 40_000.0, sek),
        ];
        let total = Money::new(100_000.0, sek);

        let alloc = sector_allocation(&positions, total, &md);

        assert_eq!(alloc.dimension, "sector");
        // Info Tech: 60,000 + 40,000*0.3 = 72,000 (72%)
        // Financials: 40,000*0.5 = 20,000 (20%)
        // Health Care: 40,000*0.2 = 8,000 (8%)
        assert_eq!(alloc.entries.len(), 3);
        assert_eq!(alloc.entries[0].key, "Information Technology");
        assert!((alloc.entries[0].market_value.amount - 72_000.0).abs() < 1e-10);
        assert_eq!(alloc.entries[1].key, "Financials");
        assert!((alloc.entries[1].market_value.amount - 20_000.0).abs() < 1e-10);
        assert_eq!(alloc.entries[2].key, "Health Care");
        assert!((alloc.entries[2].market_value.amount - 8_000.0).abs() < 1e-10);
    }
}
