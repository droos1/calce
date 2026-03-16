use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;

use calce_core::domain::account::AccountId;
use calce_core::domain::currency::Currency;
use calce_core::domain::instrument::InstrumentId;
use calce_core::domain::price::Price;
use calce_core::domain::quantity::Quantity;
use calce_core::domain::trade::Trade;
use calce_core::domain::user::UserId;

use crate::error::{DataError, DataResult};

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct UserDataRepo {
    pool: PgPool,
}

impl UserDataRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_trades(&self, user_id: &UserId) -> DataResult<Vec<Trade>> {
        let rows = sqlx::query_as::<_, TradeRow>(
            "SELECT user_id, account_id, instrument_id, quantity, price, currency, trade_date \
             FROM trades WHERE user_id = $1 ORDER BY trade_date, id",
        )
        .bind(user_id.as_str())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(TradeRow::try_into_domain)
            .collect::<DataResult<Vec<_>>>()
    }

    pub async fn get_all_trades(&self) -> DataResult<Vec<Trade>> {
        let rows = sqlx::query_as::<_, TradeRow>(
            "SELECT user_id, account_id, instrument_id, quantity, price, currency, trade_date \
             FROM trades ORDER BY user_id, trade_date, id",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(TradeRow::try_into_domain)
            .collect::<DataResult<Vec<_>>>()
    }

    pub async fn upsert_user(&self, id: &UserId, email: Option<&str>) -> DataResult<()> {
        sqlx::query(
            "INSERT INTO users (id, email) VALUES ($1, $2) \
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(id.as_str())
        .bind(email)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_account(
        &self,
        id: &AccountId,
        user_id: &UserId,
        currency: Currency,
        label: &str,
    ) -> DataResult<()> {
        sqlx::query(
            "INSERT INTO accounts (id, user_id, currency, label) VALUES ($1, $2, $3, $4) \
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(id.as_str())
        .bind(user_id.as_str())
        .bind(currency.as_str())
        .bind(label)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_users_with_trade_counts(
        &self,
    ) -> DataResult<Vec<(String, Option<String>, i64)>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: String,
            email: Option<String>,
            trade_count: Option<i64>,
        }
        let rows = sqlx::query_as::<_, Row>(
            "SELECT u.id, u.email, COUNT(t.id)::BIGINT as trade_count \
             FROM users u LEFT JOIN trades t ON u.id = t.user_id \
             GROUP BY u.id, u.email ORDER BY u.id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| (r.id, r.email, r.trade_count.unwrap_or(0)))
            .collect())
    }

    pub async fn count_users_and_trades(&self) -> DataResult<(i64, i64)> {
        let row = sqlx::query_as::<_, (i64, i64)>(
            "SELECT (SELECT COUNT(*) FROM users), (SELECT COUNT(*) FROM trades)",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn insert_trade(&self, trade: &Trade) -> DataResult<()> {
        sqlx::query(
            "INSERT INTO trades (user_id, account_id, instrument_id, quantity, price, currency, trade_date) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(trade.user_id.as_str())
        .bind(trade.account_id.as_str())
        .bind(trade.instrument_id.as_str())
        .bind(trade.quantity.value())
        .bind(trade.price.value())
        .bind(trade.currency.as_str())
        .bind(trade.date)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── CRUD operations ──────────────────────────────────────────────────

    pub async fn find_all_users(&self) -> DataResult<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, email, name, created_at FROM users ORDER BY created_at",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    pub async fn get_user(&self, id: &str) -> DataResult<User> {
        sqlx::query_as::<_, User>("SELECT id, email, name, created_at FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| DataError::NotFound(format!("user '{id}'")))
    }

    pub async fn create_user(
        &self,
        id: &str,
        email: Option<&str>,
        name: Option<&str>,
    ) -> DataResult<User> {
        sqlx::query_as::<_, User>(
            "INSERT INTO users (id, email, name) VALUES ($1, $2, $3) \
             RETURNING id, email, name, created_at",
        )
        .bind(id)
        .bind(email)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DataError::from_constraint_violation(e, "user", id))
    }

    pub async fn update_user_name(&self, id: &str, name: Option<&str>) -> DataResult<User> {
        sqlx::query_as::<_, User>(
            "UPDATE users SET name = $2 WHERE id = $1 \
             RETURNING id, email, name, created_at",
        )
        .bind(id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DataError::NotFound(format!("user '{id}'")))
    }

    /// # Errors
    ///
    /// Returns `Conflict` if the user has dependent records (accounts, trades).
    pub async fn delete_user(&self, id: &str) -> DataResult<bool> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DataError::from_constraint_violation(e, "user", id))?;
        Ok(result.rows_affected() > 0)
    }
}

#[derive(sqlx::FromRow)]
struct TradeRow {
    user_id: String,
    account_id: String,
    instrument_id: String,
    quantity: f64,
    price: f64,
    currency: String,
    trade_date: NaiveDate,
}

impl TradeRow {
    fn try_into_domain(self) -> DataResult<Trade> {
        let currency = Currency::try_new(&self.currency).map_err(|_| DataError::InvalidDbData {
            column: "currency".into(),
            value: self.currency.clone(),
            reason: "not a valid 3-letter uppercase currency code".into(),
        })?;
        Ok(Trade {
            user_id: UserId::new(self.user_id),
            account_id: AccountId::new(self.account_id),
            instrument_id: InstrumentId::new(self.instrument_id),
            quantity: Quantity::new(self.quantity),
            price: Price::new(self.price),
            currency,
            date: self.trade_date,
        })
    }
}
