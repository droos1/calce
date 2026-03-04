use chrono::NaiveDate;

use crate::domain::instrument::InstrumentId;
use crate::error::CalceResult;
use crate::services::market_data::MarketDataService;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VolatilityResult {
    pub annualized_volatility: f64,
    pub daily_volatility: f64,
    pub num_observations: usize,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

/// `#CALC_VOL`
///
/// Historical realized volatility: annualized standard deviation of
/// logarithmic daily returns.
///
/// # Errors
///
/// Returns `InsufficientData` when:
/// - fewer than 2 valid (positive) prices exist in the lookback window
/// - the earliest valid price is less than 60 days before `as_of_date`
/// - fewer than 80% of price records in the period have positive values
/// - the computed standard deviation is NaN or infinite
pub fn calculate_volatility(
    instrument: &InstrumentId,
    as_of_date: NaiveDate,
    lookback_days: u32,
    market_data: &dyn MarketDataService,
) -> CalceResult<VolatilityResult> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::instrument::InstrumentId;
    use crate::domain::price::Price;
    use crate::error::CalceError;
    use crate::services::market_data::InMemoryMarketDataService;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    /// Generate consecutive daily prices starting from `start` for `count` days.
    fn add_prices(
        md: &mut InMemoryMarketDataService,
        instrument: &InstrumentId,
        start: NaiveDate,
        values: &[f64],
    ) {
        for (i, &v) in values.iter().enumerate() {
            let d = start + chrono::Days::new(i as u64);
            md.add_price(instrument, d, Price::new(v));
        }
    }

    #[test]
    fn constant_price_gives_zero_volatility() {
        let inst = InstrumentId::new("CONST");
        let as_of = date(2025, 6, 1);
        let start = date(2025, 1, 1); // 151 days before as_of
        let mut md = InMemoryMarketDataService::new();

        // 152 days of the same price
        let prices: Vec<f64> = vec![100.0; 152];
        add_prices(&mut md, &inst, start, &prices);

        let result = calculate_volatility(&inst, as_of, 365, &md).unwrap();

        assert_eq!(result.annualized_volatility, 0.0);
        assert_eq!(result.daily_volatility, 0.0);
        assert_eq!(result.num_observations, 151); // 152 prices → 151 returns
        assert_eq!(result.start_date, start);
        assert_eq!(result.end_date, as_of);
    }

    #[test]
    fn known_volatility_from_alternating_returns() {
        // Prices alternate between 100 and 101 → log returns of ±ln(1.01)
        let inst = InstrumentId::new("ALT");
        let as_of = date(2025, 6, 1);
        let start = date(2025, 1, 1);
        let mut md = InMemoryMarketDataService::new();

        let mut prices = Vec::new();
        for i in 0..152 {
            prices.push(if i % 2 == 0 { 100.0 } else { 101.0 });
        }
        add_prices(&mut md, &inst, start, &prices);

        let result = calculate_volatility(&inst, as_of, 365, &md).unwrap();

        // 151 returns, alternating ln(1.01) and ln(100/101)
        // With odd count (151): mean is slightly nonzero, but daily vol ≈ ln(1.01)
        let r = (1.01_f64).ln();
        assert!((result.daily_volatility - r).abs() < 0.001);
        assert!((result.annualized_volatility - r * 252_f64.sqrt()).abs() < 0.02);
        assert_eq!(result.num_observations, 151);
    }

    #[test]
    fn fewer_than_two_prices_fails() {
        let inst = InstrumentId::new("LONELY");
        let mut md = InMemoryMarketDataService::new();
        md.add_price(&inst, date(2025, 1, 1), Price::new(100.0));

        let result = calculate_volatility(&inst, date(2025, 6, 1), 365, &md);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CalceError::InsufficientData { .. }
        ));
    }

    #[test]
    fn history_too_short_fails() {
        // All prices within the last 30 days — fails the 60-day depth check
        let inst = InstrumentId::new("NEW");
        let as_of = date(2025, 6, 1);
        let start = date(2025, 5, 10); // only 22 days before as_of
        let mut md = InMemoryMarketDataService::new();

        let prices: Vec<f64> = (0..23).map(|i| 100.0 + f64::from(i)).collect();
        add_prices(&mut md, &inst, start, &prices);

        let result = calculate_volatility(&inst, as_of, 365, &md);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CalceError::InsufficientData { .. }
        ));
    }

    #[test]
    fn low_completeness_fails() {
        // Insert 100 price records but make most of them zero
        let inst = InstrumentId::new("SPARSE");
        let as_of = date(2025, 6, 1);
        let start = date(2025, 1, 1);
        let mut md = InMemoryMarketDataService::new();

        // 152 records: only first 20 are valid, rest are zero → 13% completeness
        let mut prices = vec![100.0; 20];
        prices.extend(vec![0.0; 132]);
        add_prices(&mut md, &inst, start, &prices);

        let result = calculate_volatility(&inst, as_of, 365, &md);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CalceError::InsufficientData { .. }
        ));
    }

    #[test]
    fn zero_prices_are_filtered_from_returns() {
        // Mix valid prices with some zero entries — zeros should be ignored,
        // not treated as real price drops
        let inst = InstrumentId::new("GAPPY");
        let as_of = date(2025, 6, 1);
        let start = date(2025, 1, 1);
        let mut md = InMemoryMarketDataService::new();

        // 152 entries: mostly 100.0 with a few zeros sprinkled in (still >80% valid)
        let mut prices = vec![100.0; 152];
        // Set ~10% to zero (15 entries) → 90% completeness, should pass
        for i in (10..152).step_by(10) {
            prices[i] = 0.0;
        }
        add_prices(&mut md, &inst, start, &prices);

        let result = calculate_volatility(&inst, as_of, 365, &md).unwrap();

        // All valid prices are 100.0, so returns should all be zero
        assert_eq!(result.annualized_volatility, 0.0);
        assert_eq!(result.daily_volatility, 0.0);
    }

    #[test]
    fn no_prices_in_range_fails() {
        let inst = InstrumentId::new("EMPTY");
        let md = InMemoryMarketDataService::new();

        let result = calculate_volatility(&inst, date(2025, 6, 1), 365, &md);
        assert!(result.is_err());
    }
}
