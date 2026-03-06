use std::sync::Arc;

use calce_data::loader::DataLoader;

#[derive(Clone)]
pub struct AppState {
    pub loader: Arc<DataLoader>,
}
