use std::sync::Arc;

use calce_data::loader::DataLoader;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub loader: Arc<DataLoader>,
    pub pool: Option<PgPool>,
}
