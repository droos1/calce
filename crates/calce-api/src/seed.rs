use calce_core::domain::account::AccountId;
use calce_core::domain::currency::Currency;
use calce_core::domain::fx_rate::FxRate;
use calce_core::domain::instrument::InstrumentId;
use calce_core::domain::price::Price;
use calce_core::domain::quantity::Quantity;
use calce_core::domain::trade::Trade;
use calce_core::domain::user::UserId;
use calce_core::services::market_data::InMemoryMarketDataService;
use calce_core::services::user_data::InMemoryUserDataService;
use chrono::NaiveDate;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid seed date")
}

pub fn seed_market_data() -> InMemoryMarketDataService {
    let mut svc = InMemoryMarketDataService::new();

    let usd = Currency::new("USD");
    let eur = Currency::new("EUR");
    let sek = Currency::new("SEK");
    let aapl = InstrumentId::new("AAPL");
    let vow3 = InstrumentId::new("VOW3");

    let today = date(2025, 3, 15);
    let day_ago = date(2025, 3, 14);
    let week_ago = date(2025, 3, 8);
    let year_ago = date(2024, 3, 15);
    let prev_year_end = date(2024, 12, 31);

    // AAPL prices (USD)
    svc.add_price(&aapl, today, Price::new(200.0));
    svc.add_price(&aapl, day_ago, Price::new(198.0));
    svc.add_price(&aapl, week_ago, Price::new(190.0));
    svc.add_price(&aapl, year_ago, Price::new(160.0));
    svc.add_price(&aapl, prev_year_end, Price::new(180.0));

    // VOW3 prices (EUR)
    svc.add_price(&vow3, today, Price::new(120.0));
    svc.add_price(&vow3, day_ago, Price::new(119.0));
    svc.add_price(&vow3, week_ago, Price::new(115.0));
    svc.add_price(&vow3, year_ago, Price::new(100.0));
    svc.add_price(&vow3, prev_year_end, Price::new(110.0));

    // FX rates at each date
    for d in [today, day_ago, week_ago, year_ago, prev_year_end] {
        svc.add_fx_rate(FxRate::new(usd, sek, 10.5), d);
        svc.add_fx_rate(FxRate::new(eur, sek, 11.2), d);
    }

    svc
}

pub fn seed_user_data() -> InMemoryUserDataService {
    let mut svc = InMemoryUserDataService::new();

    let alice = UserId::new("alice");
    let acct_usd = AccountId::new("alice-usd");
    let acct_eur = AccountId::new("alice-eur");
    let usd = Currency::new("USD");
    let eur = Currency::new("EUR");

    svc.add_trade(Trade {
        user_id: alice.clone(),
        account_id: acct_usd,
        instrument_id: InstrumentId::new("AAPL"),
        quantity: Quantity::new(80.0),
        price: Price::new(150.0),
        currency: usd,
        date: date(2024, 1, 10),
    });

    svc.add_trade(Trade {
        user_id: alice,
        account_id: acct_eur,
        instrument_id: InstrumentId::new("VOW3"),
        quantity: Quantity::new(50.0),
        price: Price::new(95.0),
        currency: eur,
        date: date(2024, 2, 15),
    });

    svc
}
