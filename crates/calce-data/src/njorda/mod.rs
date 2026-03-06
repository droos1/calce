pub mod cache;
pub mod repo;
pub mod types;

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{Datelike, NaiveDate};

use calce_core::domain::currency::Currency;
use calce_core::domain::instrument::InstrumentId;
use calce_core::services::market_data::InMemoryMarketDataService;

use types::{CacheMetadata, CachedMarketData};

#[derive(Debug, thiserror::Error)]
pub enum NjordaError {
    #[error("Legacy DB error: {0}")]
    Database(sqlx::Error),

    #[error("Cache I/O error: {0}")]
    CacheIo(std::io::Error),

    #[error("Cache format error: {0}")]
    CacheFormat(String),

    #[error("Invalid FX ticker: {0}")]
    InvalidFxTicker(String),

    #[error("Invalid currency code: {0}")]
    InvalidCurrency(String),
}

pub struct NjordaLoader {
    repo: repo::NjordaRepo,
}

impl NjordaLoader {
    /// # Errors
    ///
    /// Returns `NjordaError::Database` if the connection fails.
    pub async fn connect(password: &str) -> Result<Self, NjordaError> {
        let repo = repo::NjordaRepo::connect(password).await?;
        Ok(Self { repo })
    }

    /// Fetch all market data in the date range and return it as a `CachedMarketData`.
    ///
    /// # Errors
    ///
    /// Returns `NjordaError::Database` on query failure.
    /// Returns `NjordaError::InvalidFxTicker` on malformed FX tickers.
    pub async fn fetch(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<CachedMarketData, NjordaError> {
        tracing::info!("Fetching active tickers from {from} to {to}");
        let all_tickers = self.repo.fetch_active_tickers(from, to).await?;
        tracing::info!("Found {} active tickers", all_tickers.len());

        // Separate FX tickers from instrument tickers
        let mut fx_tickers = Vec::new();
        let mut instrument_tickers = Vec::new();
        for ticker in &all_tickers {
            if ticker.contains('/') {
                fx_tickers.push(ticker.clone());
            } else {
                instrument_tickers.push(ticker.clone());
            }
        }

        tracing::info!(
            "Fetching prices for {} instruments and {} FX pairs",
            instrument_tickers.len(),
            fx_tickers.len()
        );

        let (prices, fx_rates, instruments) = tokio::try_join!(
            self.repo.fetch_prices(&instrument_tickers, from, to),
            self.repo.fetch_fx_rates(&fx_tickers, from, to),
            self.repo.fetch_instruments(&instrument_tickers),
        )?;

        tracing::info!(
            "Fetched {} prices, {} FX rates, {} instruments",
            prices.len(),
            fx_rates.len(),
            instruments.len()
        );

        Ok(CachedMarketData {
            metadata: CacheMetadata {
                fetched_at: chrono::Utc::now(),
                date_from: from,
                date_to: to,
                price_count: prices.len(),
                fx_rate_count: fx_rates.len(),
                instrument_count: instruments.len(),
            },
            prices,
            fx_rates,
            instruments,
        })
    }
}

/// Build an `InMemoryMarketDataService` from cached data, with forward-fill
/// for weekends/holidays.
///
/// # Errors
///
/// Returns `NjordaError::InvalidCurrency` if a cached currency code is invalid.
#[allow(clippy::cast_sign_loss)]
pub fn build_service(cached: &CachedMarketData) -> Result<InMemoryMarketDataService, NjordaError> {
    let from = cached.metadata.date_from;
    let to = cached.metadata.date_to;
    let base_day = from.num_days_from_ce();
    let num_days = (to.num_days_from_ce() - base_day + 1) as usize;

    // Group raw prices by ticker
    let mut prices_by_ticker: BTreeMap<&str, BTreeMap<NaiveDate, f64>> = BTreeMap::new();
    for p in &cached.prices {
        prices_by_ticker
            .entry(&p.ticker)
            .or_default()
            .insert(p.date, p.close);
    }

    // Scatter raw prices into dense arrays, then forward-fill
    let mut dense_prices: HashMap<InstrumentId, Vec<f64>> =
        HashMap::with_capacity(prices_by_ticker.len());
    for (ticker, date_map) in &prices_by_ticker {
        let mut arr = vec![f64::NAN; num_days];
        for (&date, &close) in date_map {
            let idx = (date.num_days_from_ce() - base_day) as usize;
            if idx < num_days {
                arr[idx] = close;
            }
        }
        // Forward-fill: carry last known price through gaps
        let mut last = f64::NAN;
        for v in &mut arr {
            if v.is_nan() {
                if !last.is_nan() {
                    *v = last;
                }
            } else {
                last = *v;
            }
        }
        dense_prices.insert(InstrumentId::new(*ticker), arr);
    }

    // Group FX rates by (from, to) pair
    let mut fx_by_pair: BTreeMap<(&str, &str), BTreeMap<NaiveDate, f64>> = BTreeMap::new();
    for r in &cached.fx_rates {
        fx_by_pair
            .entry((&r.from, &r.to))
            .or_default()
            .insert(r.date, r.rate);
    }

    // Collect all unique currency pairs to detect inverses
    let mut currency_pairs_seen: HashSet<(String, String)> = HashSet::new();
    for r in &cached.fx_rates {
        currency_pairs_seen.insert((r.from.clone(), r.to.clone()));
    }

    // Scatter raw FX rates into dense arrays, then forward-fill (direct + inverse)
    let mut dense_fx: HashMap<(Currency, Currency), Vec<f64>> = HashMap::new();
    for ((from_str, to_str), date_map) in &fx_by_pair {
        let from_ccy = Currency::try_new(from_str)
            .map_err(|_| NjordaError::InvalidCurrency((*from_str).to_string()))?;
        let to_ccy = Currency::try_new(to_str)
            .map_err(|_| NjordaError::InvalidCurrency((*to_str).to_string()))?;

        let need_inverse = {
            let inverse_key = (to_str.to_string(), from_str.to_string());
            !currency_pairs_seen.contains(&inverse_key)
        };

        // Direct rate array
        let mut arr = vec![f64::NAN; num_days];
        for (&date, &rate) in date_map {
            let idx = (date.num_days_from_ce() - base_day) as usize;
            if idx < num_days {
                arr[idx] = rate;
            }
        }
        let mut last = f64::NAN;
        for v in &mut arr {
            if v.is_nan() {
                if !last.is_nan() {
                    *v = last;
                }
            } else {
                last = *v;
            }
        }

        if need_inverse {
            let inverse: Vec<f64> = arr.iter().map(|&r| 1.0 / r).collect();
            dense_fx.insert((to_ccy, from_ccy), inverse);
        }

        dense_fx.insert((from_ccy, to_ccy), arr);
    }

    Ok(InMemoryMarketDataService::from_dense(
        base_day,
        num_days,
        dense_prices,
        dense_fx,
    ))
}

/// Print a human-readable summary of cached market data.
pub fn print_summary(cached: &CachedMarketData) {
    let m = &cached.metadata;
    println!("Njorda market data cache summary:");
    println!("  Date range: {} to {}", m.date_from, m.date_to);
    println!("  Fetched at: {}", m.fetched_at);
    println!("  Instruments: {}", m.instrument_count);
    println!("  Price points: {}", m.price_count);
    println!("  FX rate points: {}", m.fx_rate_count);
}
