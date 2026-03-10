use calce_core::domain::user::UserId;
use calce_core::error::CalceError;

pub type DataResult<T> = Result<T, DataError>;

#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("Unauthorized: user {requester} cannot access data for user {target}")]
    Unauthorized { requester: UserId, target: UserId },

    #[error("No trades found for user {0}")]
    NoTradesFound(UserId),

    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("{0}")]
    Calc(#[from] CalceError),

    #[error("Invalid data from DB: column {column}, value {value}: {reason}")]
    InvalidDbData {
        column: String,
        value: String,
        reason: String,
    },
}
