use std::collections::HashMap;

use chrono::NaiveDate;
use sqlx::PgPool;

use calce_core::domain::currency::Currency;
use calce_core::domain::fx_rate::FxRate;
use calce_core::domain::instrument::InstrumentId;
use calce_core::domain::price::Price;

use crate::error::{DataError, DataResult};

fn parse_currency(column: &str, value: String) -> DataResult<Currency> {
    Currency::try_new(&value).map_err(|_| DataError::InvalidDbData {
        column: column.into(),
        value,
        reason: "not a valid 3-letter uppercase currency code".into(),
    })
}

pub struct MarketDataRepo {
    pool: PgPool,
}

impl MarketDataRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_price(
        &self,
        instrument: &InstrumentId,
        date: NaiveDate,
    ) -> DataResult<Option<Price>> {
        let row = sqlx::query_scalar::<_, f64>(
            "SELECT p.price FROM prices p \
             JOIN instruments i ON p.instrument_id = i.id \
             WHERE i.ticker = $1 AND p.price_date = $2",
        )
        .bind(instrument.as_str())
        .bind(date)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Price::new))
    }

    pub async fn get_price_history(
        &self,
        instrument: &InstrumentId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> DataResult<Vec<(NaiveDate, Price)>> {
        let rows = sqlx::query_as::<_, (NaiveDate, f64)>(
            "SELECT p.price_date, p.price FROM prices p \
             JOIN instruments i ON p.instrument_id = i.id \
             WHERE i.ticker = $1 AND p.price_date >= $2 AND p.price_date <= $3 \
             ORDER BY p.price_date",
        )
        .bind(instrument.as_str())
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(d, p)| (d, Price::new(p))).collect())
    }

    pub async fn get_fx_rate(
        &self,
        from: Currency,
        to: Currency,
        date: NaiveDate,
    ) -> DataResult<Option<FxRate>> {
        if from == to {
            return Ok(Some(FxRate::identity(from)));
        }
        let row = sqlx::query_scalar::<_, f64>(
            "SELECT rate FROM fx_rates \
             WHERE from_currency = $1 AND to_currency = $2 AND rate_date = $3",
        )
        .bind(from.as_str())
        .bind(to.as_str())
        .bind(date)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| FxRate::new(from, to, r)))
    }

    pub async fn get_prices_batch(
        &self,
        instruments: &[InstrumentId],
        date: NaiveDate,
    ) -> DataResult<HashMap<InstrumentId, Price>> {
        if instruments.is_empty() {
            return Ok(HashMap::new());
        }
        let tickers: Vec<&str> = instruments.iter().map(InstrumentId::as_str).collect();
        let rows = sqlx::query_as::<_, (String, f64)>(
            "SELECT i.ticker, p.price FROM prices p \
             JOIN instruments i ON p.instrument_id = i.id \
             WHERE i.ticker = ANY($1) AND p.price_date = $2",
        )
        .bind(&tickers)
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, p)| (InstrumentId::new(id), Price::new(p)))
            .collect())
    }

    pub async fn get_fx_rates_batch(
        &self,
        pairs: &[(Currency, Currency)],
        date: NaiveDate,
    ) -> DataResult<HashMap<(Currency, Currency), FxRate>> {
        if pairs.is_empty() {
            return Ok(HashMap::new());
        }

        let mut result = HashMap::new();

        // Add identity rates
        for &(from, to) in pairs {
            if from == to {
                result.insert((from, to), FxRate::identity(from));
            }
        }

        let non_identity: Vec<_> = pairs.iter().filter(|(f, t)| f != t).collect();
        if non_identity.is_empty() {
            return Ok(result);
        }

        let froms: Vec<&str> = non_identity.iter().map(|(f, _)| f.as_str()).collect();
        let tos: Vec<&str> = non_identity.iter().map(|(_, t)| t.as_str()).collect();

        // Query all needed pairs at once using unnest to pair from/to arrays
        let rows = sqlx::query_as::<_, (String, String, f64)>(
            "SELECT from_currency, to_currency, rate FROM fx_rates \
             WHERE (from_currency, to_currency) IN (SELECT * FROM unnest($1::text[], $2::text[])) \
             AND rate_date = $3",
        )
        .bind(&froms)
        .bind(&tos)
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        for (from_str, to_str, rate) in rows {
            let from = parse_currency("from_currency", from_str)?;
            let to = parse_currency("to_currency", to_str)?;
            result.insert((from, to), FxRate::new(from, to, rate));
        }

        Ok(result)
    }

    pub async fn get_fx_rate_history(
        &self,
        from_ccy: Currency,
        to_ccy: Currency,
        date_from: NaiveDate,
        date_to: NaiveDate,
    ) -> DataResult<Vec<(NaiveDate, FxRate)>> {
        let rows = sqlx::query_as::<_, (NaiveDate, f64)>(
            "SELECT rate_date, rate FROM fx_rates \
             WHERE from_currency = $1 AND to_currency = $2 \
             AND rate_date >= $3 AND rate_date <= $4 \
             ORDER BY rate_date",
        )
        .bind(from_ccy.as_str())
        .bind(to_ccy.as_str())
        .bind(date_from)
        .bind(date_to)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(d, r)| (d, FxRate::new(from_ccy, to_ccy, r)))
            .collect())
    }

    pub async fn get_price_history_batch(
        &self,
        instruments: &[InstrumentId],
        from: NaiveDate,
        to: NaiveDate,
    ) -> DataResult<HashMap<InstrumentId, Vec<(NaiveDate, Price)>>> {
        if instruments.is_empty() {
            return Ok(HashMap::new());
        }
        let tickers: Vec<&str> = instruments.iter().map(InstrumentId::as_str).collect();
        let rows = sqlx::query_as::<_, (String, NaiveDate, f64)>(
            "SELECT i.ticker, p.price_date, p.price FROM prices p \
             JOIN instruments i ON p.instrument_id = i.id \
             WHERE i.ticker = ANY($1) AND p.price_date >= $2 AND p.price_date <= $3 \
             ORDER BY i.ticker, p.price_date",
        )
        .bind(&tickers)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        let mut result: HashMap<InstrumentId, Vec<(NaiveDate, Price)>> = HashMap::new();
        for (id, date, price) in rows {
            result
                .entry(InstrumentId::new(id))
                .or_default()
                .push((date, Price::new(price)));
        }
        Ok(result)
    }

    pub async fn get_fx_rate_history_batch(
        &self,
        pairs: &[(Currency, Currency)],
        from: NaiveDate,
        to: NaiveDate,
    ) -> DataResult<Vec<(NaiveDate, FxRate)>> {
        if pairs.is_empty() {
            return Ok(Vec::new());
        }

        let non_identity: Vec<_> = pairs.iter().filter(|(f, t)| f != t).collect();
        if non_identity.is_empty() {
            return Ok(Vec::new());
        }

        let froms: Vec<&str> = non_identity.iter().map(|(f, _)| f.as_str()).collect();
        let tos: Vec<&str> = non_identity.iter().map(|(_, t)| t.as_str()).collect();

        let rows = sqlx::query_as::<_, (String, String, NaiveDate, f64)>(
            "SELECT from_currency, to_currency, rate_date, rate FROM fx_rates \
             WHERE (from_currency, to_currency) IN (SELECT * FROM unnest($1::text[], $2::text[])) \
             AND rate_date >= $3 AND rate_date <= $4 \
             ORDER BY rate_date",
        )
        .bind(&froms)
        .bind(&tos)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::with_capacity(rows.len());
        for (from_str, to_str, date, rate) in rows {
            let from_ccy = parse_currency("from_currency", from_str)?;
            let to_ccy = parse_currency("to_currency", to_str)?;
            result.push((date, FxRate::new(from_ccy, to_ccy, rate)));
        }
        Ok(result)
    }

    pub async fn get_all_prices(&self) -> DataResult<Vec<(InstrumentId, NaiveDate, Price)>> {
        let rows = sqlx::query_as::<_, (String, NaiveDate, f64)>(
            "SELECT i.ticker, p.price_date, p.price FROM prices p \
             JOIN instruments i ON p.instrument_id = i.id \
             ORDER BY i.ticker, p.price_date",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, d, p)| (InstrumentId::new(id), d, Price::new(p)))
            .collect())
    }

    pub async fn get_all_fx_rates(&self) -> DataResult<Vec<(NaiveDate, FxRate)>> {
        let rows = sqlx::query_as::<_, (String, String, NaiveDate, f64)>(
            "SELECT from_currency, to_currency, rate_date, rate FROM fx_rates ORDER BY rate_date",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|(from_str, to_str, date, rate)| {
                let from = parse_currency("from_currency", from_str)?;
                let to = parse_currency("to_currency", to_str)?;
                Ok((date, FxRate::new(from, to, rate)))
            })
            .collect()
    }

    pub async fn list_instruments(&self) -> DataResult<Vec<(String, String, Option<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
            "SELECT ticker, currency, name FROM instruments ORDER BY ticker",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn count_market_data(&self) -> DataResult<(i64, i64, i64)> {
        let row = sqlx::query_as::<_, (i64, i64, i64)>(
            "SELECT \
             (SELECT COUNT(*) FROM instruments), \
             (SELECT COUNT(*) FROM prices), \
             (SELECT COUNT(*) FROM fx_rates)",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn insert_price(
        &self,
        instrument: &InstrumentId,
        date: NaiveDate,
        price: Price,
    ) -> DataResult<()> {
        sqlx::query(
            "INSERT INTO prices (instrument_id, price_date, price) \
             VALUES ((SELECT id FROM instruments WHERE ticker = $1), $2, $3) \
             ON CONFLICT ON CONSTRAINT uq_prices_instrument_date DO UPDATE SET price = EXCLUDED.price",
        )
        .bind(instrument.as_str())
        .bind(date)
        .bind(price.value())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_fx_rate(&self, rate: &FxRate, date: NaiveDate) -> DataResult<()> {
        sqlx::query(
            "INSERT INTO fx_rates (from_currency, to_currency, rate_date, rate) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (from_currency, to_currency, rate_date) DO UPDATE SET rate = EXCLUDED.rate",
        )
        .bind(rate.from.as_str())
        .bind(rate.to.as_str())
        .bind(date)
        .bind(rate.rate)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_instrument(&self, id: &InstrumentId, currency: Currency) -> DataResult<()> {
        sqlx::query(
            "INSERT INTO instruments (ticker, currency) VALUES ($1, $2) \
             ON CONFLICT (ticker) DO NOTHING",
        )
        .bind(id.as_str())
        .bind(currency.as_str())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
