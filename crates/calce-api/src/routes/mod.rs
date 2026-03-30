mod api_keys;
pub mod auth;
mod calc;
mod organizations;
mod simulator;
mod users;

use axum::Router;

use crate::state::AppState;

pub fn calc_routes() -> Router<AppState> {
    calc::routes()
}

pub fn user_routes() -> Router<AppState> {
    users::routes()
}

pub fn organization_routes() -> Router<AppState> {
    organizations::routes()
}

pub fn auth_routes() -> Router<AppState> {
    auth::routes()
}

pub fn api_key_routes() -> Router<AppState> {
    api_keys::routes()
}

pub fn simulator_routes() -> Router<AppState> {
    simulator::routes()
}
