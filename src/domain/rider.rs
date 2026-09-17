use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::referral::generate_referral_code;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rider {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub referral_code: String,
    pub referred_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Rider {
    pub fn new(
        name: String,
        email: String,
        password_hash: String,
        referred_by: Option<String>,
    ) -> Self {
        Rider {
            id: Uuid::new_v4(),
            name,
            email,
            password_hash,
            referral_code: generate_referral_code(),
            referred_by,
            created_at: Utc::now(),
        }
    }
}
