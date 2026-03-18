use chrono::NaiveDate;

use calce_core::calc::aggregation::aggregate_positions;
use calce_core::calc::market_value::value_positions;
use calce_core::context::CalculationContext;
use calce_core::domain::account::AccountId;
use calce_core::domain::currency::Currency;
use calce_core::domain::fx_rate::FxRate;
use calce_core::domain::instrument::{InstrumentId, InstrumentType};
use calce_core::domain::price::Price;
use calce_core::domain::quantity::Quantity;
use calce_core::domain::trade::Trade;
use calce_core::domain::user::UserId;
use calce_core::reports::portfolio::portfolio_report;
use calce_core::services::test_market_data::TestMarketData;

fn setup_multi_currency_scenario() -> (TestMarketData, Vec<Trade>, NaiveDate) {
    let alice = UserId::new("alice");
    let acct_usd = AccountId::new(1);
    let acct_eur = AccountId::new(2);
    let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
    let usd = Currency::new("USD");
    let eur = Currency::new("EUR");
    let sek = Currency::new("SEK");
    let aapl = InstrumentId::new("AAPL");
    let vow3 = InstrumentId::new("VOW3");
    let spy = InstrumentId::new("SPY");

    let mut market_data = TestMarketData::new();
    market_data.add_price(&aapl, date, Price::new(150.0));
    market_data.add_price(&vow3, date, Price::new(120.0));
    market_data.add_price(&spy, date, Price::new(500.0));
    market_data.add_fx_rate(FxRate::new(usd, sek, 10.5), date);
    market_data.add_fx_rate(FxRate::new(eur, sek, 11.4), date);

    market_data.add_instrument_type(&aapl, InstrumentType::Stock);
    market_data.add_instrument_type(&vow3, InstrumentType::Stock);
    market_data.add_instrument_type(&spy, InstrumentType::Etf);

    let trades = vec![
        // Alice buys 100 AAPL, sells 20 → net 80
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
            user_id: alice.clone(),
            account_id: acct_usd,
            instrument_id: aapl,
            quantity: Quantity::new(-20.0),
            price: Price::new(155.0),
            currency: usd,
            date,
        },
        // Alice buys 50 VOW3
        Trade {
            user_id: alice.clone(),
            account_id: acct_eur,
            instrument_id: vow3,
            quantity: Quantity::new(50.0),
            price: Price::new(115.0),
            currency: eur,
            date,
        },
        // Alice buys 10 SPY
        Trade {
            user_id: alice,
            account_id: acct_usd,
            instrument_id: spy,
            quantity: Quantity::new(10.0),
            price: Price::new(490.0),
            currency: usd,
            date,
        },
    ];

    (market_data, trades, date)
}

// ---------------------------------------------------------------------------
// End-to-end: aggregation + valuation
// ---------------------------------------------------------------------------

#[test]
fn multi_currency_portfolio() {
    let (market_data, trades, date) = setup_multi_currency_scenario();
    let sek = Currency::new("SEK");

    let positions = aggregate_positions(&trades, date).unwrap();
    let ctx = CalculationContext::new(sek, date);
    let result = value_positions(&positions, &ctx, &market_data)
        .expect("should succeed")
        .value;

    // AAPL: 80 * 150 = 12,000 USD → 12,000 * 10.5 = 126,000 SEK
    // VOW3: 50 * 120 = 6,000 EUR → 6,000 * 11.4 = 68,400 SEK
    // SPY:  10 * 500 = 5,000 USD → 5,000 * 10.5 = 52,500 SEK
    // Total: 126,000 + 68,400 + 52,500 = 246,900 SEK
    assert_eq!(result.positions.len(), 3);
    assert_eq!(result.total.amount, 246_900.0);
    assert_eq!(result.total.currency, sek);

    let aapl_pos = &result.positions[0];
    assert_eq!(aapl_pos.instrument_id.as_str(), "AAPL");
    assert_eq!(aapl_pos.quantity.value(), 80.0);
    assert_eq!(aapl_pos.market_value.amount, 12_000.0);
    assert_eq!(aapl_pos.market_value_base.amount, 126_000.0);

    let spy_pos = &result.positions[1];
    assert_eq!(spy_pos.instrument_id.as_str(), "SPY");
    assert_eq!(spy_pos.quantity.value(), 10.0);
    assert_eq!(spy_pos.market_value.amount, 5_000.0);
    assert_eq!(spy_pos.market_value_base.amount, 52_500.0);

    let vow3_pos = &result.positions[2];
    assert_eq!(vow3_pos.instrument_id.as_str(), "VOW3");
    assert_eq!(vow3_pos.quantity.value(), 50.0);
    assert_eq!(vow3_pos.market_value.amount, 6_000.0);
    assert_eq!(vow3_pos.market_value_base.amount, 68_400.0);
}

#[test]
fn retroactive_calculation() {
    let alice = UserId::new("alice");
    let acct = AccountId::new(1);
    let usd = Currency::new("USD");
    let aapl = InstrumentId::new("AAPL");

    let early = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();
    let late = NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();

    let mut market_data = TestMarketData::new();
    market_data.add_price(&aapl, early, Price::new(140.0));

    let trades = vec![
        Trade {
            user_id: alice.clone(),
            account_id: acct,
            instrument_id: aapl.clone(),
            quantity: Quantity::new(50.0),
            price: Price::new(135.0),
            currency: usd,
            date: early,
        },
        Trade {
            user_id: alice,
            account_id: acct,
            instrument_id: aapl,
            quantity: Quantity::new(30.0),
            price: Price::new(145.0),
            currency: usd,
            date: late,
        },
    ];

    let positions = aggregate_positions(&trades, early).unwrap();
    let ctx = CalculationContext::new(usd, early);
    let result = value_positions(&positions, &ctx, &market_data)
        .expect("should succeed")
        .value;

    assert_eq!(result.positions.len(), 1);
    assert_eq!(result.positions[0].quantity.value(), 50.0);
    // 50 * 140 = 7,000 USD (same currency, no FX)
    assert_eq!(result.total.amount, 7_000.0);
}

// ---------------------------------------------------------------------------
// Calculation function tests — no auth, no user data, just positions + market data
// ---------------------------------------------------------------------------

#[test]
fn value_positions_multi_currency() {
    let usd = Currency::new("USD");
    let eur = Currency::new("EUR");
    let sek = Currency::new("SEK");
    let aapl = InstrumentId::new("AAPL");
    let vow3 = InstrumentId::new("VOW3");
    let spy = InstrumentId::new("SPY");
    let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

    let mut market_data = TestMarketData::new();
    market_data.add_price(&aapl, date, Price::new(150.0));
    market_data.add_price(&vow3, date, Price::new(120.0));
    market_data.add_price(&spy, date, Price::new(500.0));
    market_data.add_fx_rate(FxRate::new(usd, sek, 10.5), date);
    market_data.add_fx_rate(FxRate::new(eur, sek, 11.4), date);

    let positions = vec![
        calce_core::domain::position::Position {
            instrument_id: aapl,
            quantity: Quantity::new(80.0),
            currency: usd,
        },
        calce_core::domain::position::Position {
            instrument_id: vow3,
            quantity: Quantity::new(50.0),
            currency: eur,
        },
        calce_core::domain::position::Position {
            instrument_id: spy,
            quantity: Quantity::new(10.0),
            currency: usd,
        },
    ];
    let ctx = CalculationContext::new(sek, date);

    let result = value_positions(&positions, &ctx, &market_data)
        .unwrap()
        .value;

    // AAPL: 80 * 150 * 10.5 = 126,000
    // VOW3: 50 * 120 * 11.4 = 68,400
    // SPY:  10 * 500 * 10.5 = 52,500
    // Total: 246,900
    assert_eq!(result.total.amount, 246_900.0);
    assert_eq!(result.total.currency, sek);
}

#[test]
fn aggregate_then_value() {
    let usd = Currency::new("USD");
    let aapl = InstrumentId::new("AAPL");
    let alice = UserId::new("alice");
    let acct = AccountId::new(1);
    let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

    let trades = vec![
        Trade {
            user_id: alice.clone(),
            account_id: acct,
            instrument_id: aapl.clone(),
            quantity: Quantity::new(100.0),
            price: Price::new(145.0),
            currency: usd,
            date,
        },
        Trade {
            user_id: alice,
            account_id: acct,
            instrument_id: aapl.clone(),
            quantity: Quantity::new(-40.0),
            price: Price::new(155.0),
            currency: usd,
            date,
        },
    ];

    // Step 1: aggregate trades into positions
    let positions = aggregate_positions(&trades, date).unwrap();
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].quantity.value(), 60.0);

    // Step 2: value positions
    let mut market_data = TestMarketData::new();
    market_data.add_price(&aapl, date, Price::new(150.0));
    let ctx = CalculationContext::new(usd, date);

    let result = value_positions(&positions, &ctx, &market_data)
        .unwrap()
        .value;
    assert_eq!(result.total.amount, 9_000.0); // 60 * 150
}

// ---------------------------------------------------------------------------
// Portfolio report — full pipeline integration test
// ---------------------------------------------------------------------------

#[test]
fn portfolio_report_integration() {
    let alice = UserId::new("alice");
    let acct = AccountId::new(1);
    let usd = Currency::new("USD");
    let sek = Currency::new("SEK");
    let aapl = InstrumentId::new("AAPL");

    let today = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
    let day_ago = today - chrono::Days::new(1);
    let week_ago = today - chrono::Days::new(7);
    let year_ago = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
    let prev_year_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    let trade_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    let mut market_data = TestMarketData::new();
    market_data.add_price(&aapl, today, Price::new(200.0));
    market_data.add_price(&aapl, day_ago, Price::new(198.0));
    market_data.add_price(&aapl, week_ago, Price::new(190.0));
    market_data.add_price(&aapl, year_ago, Price::new(160.0));
    market_data.add_price(&aapl, prev_year_end, Price::new(180.0));
    for date in [today, day_ago, week_ago, year_ago, prev_year_end] {
        market_data.add_fx_rate(FxRate::new(usd, sek, 10.0), date);
    }

    let trades = vec![Trade {
        user_id: alice,
        account_id: acct,
        instrument_id: aapl,
        quantity: Quantity::new(100.0),
        price: Price::new(150.0),
        currency: usd,
        date: trade_date,
    }];

    let ctx = CalculationContext::new(sek, today);
    let outcome = portfolio_report(&trades, &ctx, &market_data).expect("report should succeed");
    let report = &outcome.value;

    // Market value: 100 * 200 * 10 = 200,000 SEK
    assert_eq!(report.market_value.total.amount, 200_000.0);
    assert_eq!(report.market_value.positions.len(), 1);

    // Value changes
    assert_eq!(report.value_changes.market_value.amount, 200_000.0);
    assert_eq!(report.value_changes.daily.change.amount, 2_000.0);
    assert_eq!(report.value_changes.weekly.change.amount, 10_000.0);
    assert_eq!(report.value_changes.yearly.change.amount, 40_000.0);
    assert_eq!(report.value_changes.ytd.change.amount, 20_000.0);

    // Type allocation: no instrument types set, so all go to Other
    assert_eq!(report.type_allocation.entries.len(), 1);
    assert_eq!(
        report.type_allocation.entries[0].instrument_type,
        InstrumentType::Other
    );
}

// ---------------------------------------------------------------------------
// Type allocation in portfolio report
// ---------------------------------------------------------------------------

#[test]
fn portfolio_report_type_allocation() {
    let (_market_data, trades, date) = setup_multi_currency_scenario();
    let sek = Currency::new("SEK");

    // Need value change reference dates
    let day_ago = date - chrono::Days::new(1);
    let week_ago = date - chrono::Days::new(7);
    let year_ago = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let prev_year_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    // The setup has AAPL/SPY prices on `date` but we need historical prices
    // for value change. Use a standalone TestMarketData that has everything.
    let aapl = InstrumentId::new("AAPL");
    let vow3 = InstrumentId::new("VOW3");
    let spy = InstrumentId::new("SPY");
    let usd = Currency::new("USD");
    let eur = Currency::new("EUR");

    let mut md = TestMarketData::new();
    md.add_price(&aapl, date, Price::new(150.0));
    md.add_price(&vow3, date, Price::new(120.0));
    md.add_price(&spy, date, Price::new(500.0));
    for d in [date, day_ago, week_ago, year_ago, prev_year_end] {
        md.add_price(&aapl, d, Price::new(150.0));
        md.add_price(&vow3, d, Price::new(120.0));
        md.add_price(&spy, d, Price::new(500.0));
        md.add_fx_rate(FxRate::new(usd, sek, 10.5), d);
        md.add_fx_rate(FxRate::new(eur, sek, 11.4), d);
    }
    md.add_instrument_type(&aapl, InstrumentType::Stock);
    md.add_instrument_type(&vow3, InstrumentType::Stock);
    md.add_instrument_type(&spy, InstrumentType::Etf);

    let ctx = CalculationContext::new(sek, date);
    let outcome = portfolio_report(&trades, &ctx, &md).expect("report should succeed");
    let alloc = &outcome.value.type_allocation;

    // AAPL: 80 * 150 * 10.5 = 126,000 SEK (Stock)
    // VOW3: 50 * 120 * 11.4 = 68,400 SEK  (Stock)
    // SPY:  10 * 500 * 10.5 = 52,500 SEK  (Etf)
    // Total: 246,900 SEK
    // Stock = 194,400 / 246,900 ≈ 0.7874
    // Etf   = 52,500  / 246,900 ≈ 0.2126
    assert_eq!(alloc.entries.len(), 2);

    // Sorted by descending weight: Stock first
    assert_eq!(alloc.entries[0].instrument_type, InstrumentType::Stock);
    assert_eq!(alloc.entries[0].market_value.amount, 194_400.0);
    assert!((alloc.entries[0].weight - 194_400.0 / 246_900.0).abs() < 1e-10);

    assert_eq!(alloc.entries[1].instrument_type, InstrumentType::Etf);
    assert_eq!(alloc.entries[1].market_value.amount, 52_500.0);
    assert!((alloc.entries[1].weight - 52_500.0 / 246_900.0).abs() < 1e-10);

    // Weights sum to ~1.0
    let sum: f64 = alloc.entries.iter().map(|e| e.weight).sum();
    assert!((sum - 1.0).abs() < 1e-10);
}
