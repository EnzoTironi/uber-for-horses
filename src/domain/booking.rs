use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BookingStatus {
    Requested,
    Confirmed,
    Declined,
    Cancelled,
    Completed,
}

impl BookingStatus {
    /// Legal state transitions:
    /// Requested -> {Confirmed, Declined, Cancelled}
    /// Confirmed -> {Completed, Cancelled}
    /// all other states are terminal (no outgoing transitions).
    pub fn can_transition_to(&self, next: BookingStatus) -> bool {
        use BookingStatus::*;
        matches!(
            (self, next),
            (Requested, Confirmed)
                | (Requested, Declined)
                | (Requested, Cancelled)
                | (Confirmed, Completed)
                | (Confirmed, Cancelled)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Booking {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub rider_id: Uuid,
    pub time_slot_id: Uuid,
    pub status: BookingStatus,
    pub total_price_cents: i64,
    pub message_from_rider: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Booking {
    pub fn new(
        listing_id: Uuid,
        rider_id: Uuid,
        time_slot_id: Uuid,
        total_price_cents: i64,
        message_from_rider: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Booking {
            id: Uuid::new_v4(),
            listing_id,
            rider_id,
            time_slot_id,
            status: BookingStatus::Requested,
            total_price_cents,
            message_from_rider,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requested_can_go_to_confirmed_declined_cancelled() {
        assert!(BookingStatus::Requested.can_transition_to(BookingStatus::Confirmed));
        assert!(BookingStatus::Requested.can_transition_to(BookingStatus::Declined));
        assert!(BookingStatus::Requested.can_transition_to(BookingStatus::Cancelled));
        assert!(!BookingStatus::Requested.can_transition_to(BookingStatus::Completed));
        assert!(!BookingStatus::Requested.can_transition_to(BookingStatus::Requested));
    }

    #[test]
    fn confirmed_can_go_to_completed_cancelled() {
        assert!(BookingStatus::Confirmed.can_transition_to(BookingStatus::Completed));
        assert!(BookingStatus::Confirmed.can_transition_to(BookingStatus::Cancelled));
        assert!(!BookingStatus::Confirmed.can_transition_to(BookingStatus::Requested));
        assert!(!BookingStatus::Confirmed.can_transition_to(BookingStatus::Declined));
    }

    #[test]
    fn terminal_states_have_no_outgoing_transitions() {
        let terminal = [
            BookingStatus::Declined,
            BookingStatus::Cancelled,
            BookingStatus::Completed,
        ];
        let all = [
            BookingStatus::Requested,
            BookingStatus::Confirmed,
            BookingStatus::Declined,
            BookingStatus::Cancelled,
            BookingStatus::Completed,
        ];
        for t in terminal {
            for next in all {
                assert!(
                    !t.can_transition_to(next),
                    "{:?} should not transition to {:?}",
                    t,
                    next
                );
            }
        }
    }
}
