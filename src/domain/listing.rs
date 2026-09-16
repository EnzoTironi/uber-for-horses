use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimalKind {
    Horse,
    Carriage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub kind: AnimalKind,
    pub name: String,
    pub description: String,
    pub photo_url: String,
    pub hourly_price_cents: i64,
    pub lat: f64,
    pub lng: f64,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

impl Listing {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        owner_id: Uuid,
        kind: AnimalKind,
        name: String,
        description: String,
        photo_url: String,
        hourly_price_cents: i64,
        lat: f64,
        lng: f64,
    ) -> Self {
        Listing {
            id: Uuid::new_v4(),
            owner_id,
            kind,
            name,
            description,
            photo_url,
            hourly_price_cents,
            lat,
            lng,
            active: true,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub is_booked: bool,
}

impl TimeSlot {
    pub fn new(listing_id: Uuid, start_at: DateTime<Utc>, end_at: DateTime<Utc>) -> Self {
        TimeSlot {
            id: Uuid::new_v4(),
            listing_id,
            start_at,
            end_at,
            is_booked: false,
        }
    }

    /// Duration of the slot in hours, rounded up (ceil), minimum 1 hour.
    pub fn duration_hours_ceil(&self) -> i64 {
        let seconds = (self.end_at - self.start_at).num_seconds().max(0);
        let hours = (seconds as f64 / 3600.0).ceil() as i64;
        hours.max(1)
    }
}
