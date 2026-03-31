use std::convert::Infallible;

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::sse::{Event, Sse};
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

use crate::auth::require_admin;
use crate::error::ApiError;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/v1/events", get(events_sse))
}

#[derive(Serialize)]
struct EntityUpdate {
    #[serde(rename = "type")]
    update_type: &'static str,
    table: String,
    id: String,
}

/// EventSource doesn't support custom headers, so we also accept the JWT
/// as a `?token=` query parameter.
#[derive(Deserialize)]
struct SseQuery {
    token: Option<String>,
}

async fn events_sse(
    headers: HeaderMap,
    Query(query): Query<SseQuery>,
    State(state): State<AppState>,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>, ApiError> {
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

    let entity_pubsub = state
        .entity_pubsub
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("entity pubsub not available".into()))?;

    let sub = entity_pubsub.subscribe_all(256);

    let stream =
        tokio_stream::wrappers::ReceiverStream::new(sub.receiver).map(|event| {
            let key_str = match &event {
                calce_datastructs::pubsub::UpdateEvent::CurrentChanged { key } => key.as_str(),
                calce_datastructs::pubsub::UpdateEvent::HistoryChanged { key } => key.as_str(),
            };
            let (table, id) = key_str.split_once(':').unwrap_or(("unknown", key_str));
            let update = EntityUpdate {
                update_type: "entity",
                table: table.to_owned(),
                id: id.to_owned(),
            };
            let data = serde_json::to_string(&update).unwrap_or_default();
            Ok::<_, Infallible>(Event::default().event("update").data(data))
        });

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new().interval(std::time::Duration::from_secs(15)),
    ))
}
