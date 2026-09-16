use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl Owner {
    pub fn new(name: String, email: String, password_hash: String) -> Self {
        Owner {
            id: Uuid::new_v4(),
            name,
            email,
            password_hash,
            created_at: Utc::now(),
        }
    }
}
