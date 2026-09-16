use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl Owner {
    pub fn new(name: String, email: String) -> Self {
        Owner {
            id: Uuid::new_v4(),
            name,
            email,
            created_at: Utc::now(),
        }
    }
}
