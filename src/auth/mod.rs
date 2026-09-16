pub mod jwt;

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

use crate::api::AppState;
use crate::error::AppError;
use jwt::AuthIdentity;

/// Axum extractor that pulls a bearer JWT out of the `Authorization` header,
/// validates it, and yields the authenticated identity. Any missing, malformed
/// or expired token results in a 401 Unauthorized.
#[async_trait]
impl FromRequestParts<AppState> for AuthIdentity {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header_value = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing authorization header".into()))?;

        let token = header_value
            .strip_prefix("Bearer ")
            .or_else(|| header_value.strip_prefix("bearer "))
            .ok_or_else(|| {
                AppError::Unauthorized("authorization header must be a bearer token".into())
            })?;

        let claims = jwt::decode_token(token)?;

        Ok(AuthIdentity {
            id: claims.sub,
            role: claims.role,
        })
    }
}
