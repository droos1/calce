mod calc;
mod users;

use axum::Router;

use crate::state::AppState;

pub use calc::explorer;

pub fn calc_routes() -> Router<AppState> {
    calc::routes()
}

pub fn user_routes() -> Router<AppState> {
    users::routes()
}
