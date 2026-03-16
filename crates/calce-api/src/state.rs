use std::sync::Arc;

use calce_data::service::DataService;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub data: Arc<DataService>,
    pub pool: Option<PgPool>,
}
