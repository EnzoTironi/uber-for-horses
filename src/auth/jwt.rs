use std::env;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::Role;
use crate::error::AppError;

const DEFAULT_DEV_SECRET: &str = "dev-secret-do-not-use-in-production";
const ACCESS_TOKEN_TTL_SECONDS: i64 = 60 * 60; // 1 hour

fn jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_DEV_SECRET.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: Role,
    pub exp: i64,
}

/// The authenticated identity extracted from a validated JWT.
#[derive(Debug, Clone, Copy)]
pub struct AuthIdentity {
    pub id: Uuid,
    pub role: Role,
}

pub fn issue_token(id: Uuid, role: Role) -> Result<String, AppError> {
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(ACCESS_TOKEN_TTL_SECONDS))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims { sub: id, role, exp };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("failed to issue token: {e}")))
}

pub fn decode_token(token: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::Unauthorized(format!("invalid token: {e}")))
}
