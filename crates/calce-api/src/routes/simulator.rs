use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::sse::{Event, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

use crate::auth::{Auth, require_admin};
use crate::error::ApiError;
use crate::simulator::SimulatorStats;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/admin/simulator/start", post(start))
        .route("/v1/admin/simulator/stop", post(stop))
        .route("/v1/admin/simulator/status", get(status))
        .route("/v1/admin/simulator/events", get(events_sse))
}

async fn start(
    Auth(ctx): Auth,
    State(state): State<AppState>,
) -> Result<Json<SimulatorStats>, ApiError> {
    require_admin(&ctx)?;
    let sim = simulator(&state)?;
    sim.start().await;
    Ok(Json(sim.stats()))
}

async fn stop(
    Auth(ctx): Auth,
    State(state): State<AppState>,
) -> Result<Json<SimulatorStats>, ApiError> {
    require_admin(&ctx)?;
    let sim = simulator(&state)?;
    sim.stop().await;
    Ok(Json(sim.stats()))
}

async fn status(
    Auth(ctx): Auth,
    State(state): State<AppState>,
) -> Result<Json<SimulatorStats>, ApiError> {
    require_admin(&ctx)?;
    let sim = simulator(&state)?;
    Ok(Json(sim.stats()))
}

#[derive(Serialize)]
struct SseUpdate {
    #[serde(rename = "type")]
    update_type: &'static str,
    key: String,
    kind: &'static str,
}

/// EventSource doesn't support custom headers, so we also accept the JWT
/// as a `?token=` query parameter for the SSE endpoint.
#[derive(Deserialize)]
struct SseQuery {
    token: Option<String>,
}

async fn events_sse(
    headers: HeaderMap,
    Query(query): Query<SseQuery>,
    State(state): State<AppState>,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>, ApiError> {
    // EventSource doesn't support custom headers, so accept JWT from either
    // the Authorization header or a ?token= query parameter.
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(String::from)
        .or(query.token)
        .ok_or(ApiError::Data(
            calce_data::error::DataError::InvalidCredentials,
        ))?;

    let ctx = calce_data::auth::middleware::validate_bearer_token(
        &token,
        &state.auth_config,
        state.pool.as_ref(),
        Some(&state.api_key_cache),
    )
    .await
    .map_err(|_| ApiError::Data(calce_data::error::DataError::InvalidCredentials))?;
    require_admin(&ctx)?;

    let md = state.market_data.market_data();

    let price_pubsub = state
        .price_pubsub
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("pubsub not available".into()))?;
    let fx_pubsub = state
        .fx_pubsub
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("pubsub not available".into()))?;

    // Subscribe to all known keys.
    let instrument_ids = md.instrument_ids();
    let fx_keys = md.fx_pair_keys();

    let price_sub = price_pubsub.subscribe(&instrument_ids, 256);
    let fx_sub = fx_pubsub.subscribe(&fx_keys, 256);

    let price_stream =
        tokio_stream::wrappers::ReceiverStream::new(price_sub.receiver).map(|event| {
            let (key, kind) = match &event {
                calce_ds::pubsub::UpdateEvent::CurrentChanged { key } => {
                    (key.as_str().to_owned(), "current")
                }
                calce_ds::pubsub::UpdateEvent::HistoryChanged { key } => {
                    (key.as_str().to_owned(), "history")
                }
            };
            let update = SseUpdate {
                update_type: "price",
                key,
                kind,
            };
            let data = serde_json::to_string(&update).unwrap_or_default();
            Ok::<_, Infallible>(Event::default().event("update").data(data))
        });

    let fx_stream = tokio_stream::wrappers::ReceiverStream::new(fx_sub.receiver).map(|event| {
        let (key, kind) = match &event {
            calce_ds::pubsub::UpdateEvent::CurrentChanged { key } => {
                (format!("{}/{}", key.0.as_str(), key.1.as_str()), "current")
            }
            calce_ds::pubsub::UpdateEvent::HistoryChanged { key } => {
                (format!("{}/{}", key.0.as_str(), key.1.as_str()), "history")
            }
        };
        let update = SseUpdate {
            update_type: "fx",
            key,
            kind,
        };
        let data = serde_json::to_string(&update).unwrap_or_default();
        Ok::<_, Infallible>(Event::default().event("update").data(data))
    });

    let merged = price_stream.merge(fx_stream);

    Ok(Sse::new(merged).keep_alive(
        axum::response::sse::KeepAlive::new().interval(std::time::Duration::from_secs(15)),
    ))
}

fn simulator(state: &AppState) -> Result<&Arc<crate::simulator::Simulator>, ApiError> {
    state
        .simulator
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("simulator not available".into()))
}
