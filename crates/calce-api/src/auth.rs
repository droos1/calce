use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use calce_core::auth::{Role, SecurityContext};
use calce_core::domain::user::UserId;
use serde_json::json;

pub struct Auth(pub SecurityContext);

#[derive(Debug)]
pub enum AuthError {
    MissingUserId,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = json!({ "error": "UNAUTHORIZED", "message": "Missing X-User-Id header" });
        (StatusCode::UNAUTHORIZED, axum::Json(body)).into_response()
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Auth {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user_id = parts
            .headers
            .get("x-user-id")
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::MissingUserId)?;

        let role = parts
            .headers
            .get("x-role")
            .and_then(|v| v.to_str().ok())
            .map_or(Role::User, |r| {
                if r.eq_ignore_ascii_case("admin") {
                    Role::Admin
                } else {
                    Role::User
                }
            });

        Ok(Auth(SecurityContext::new(UserId::new(user_id), role)))
    }
}
