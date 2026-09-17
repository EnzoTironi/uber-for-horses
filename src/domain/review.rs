use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub rider_id: Uuid,
    pub listing_id: Uuid,
    pub rating: i32,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Review {
    /// Construct a new review, validating that `rating` is within the
    /// inclusive 1..=5 range.
    pub fn new(
        booking_id: Uuid,
        rider_id: Uuid,
        listing_id: Uuid,
        rating: i32,
        comment: Option<String>,
    ) -> Result<Self, AppError> {
        if !(1..=5).contains(&rating) {
            return Err(AppError::Validation(
                "rating must be between 1 and 5".into(),
            ));
        }
        Ok(Review {
            id: Uuid::new_v4(),
            booking_id,
            rider_id,
            listing_id,
            rating,
            comment,
            created_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rating_within_1_to_5_is_accepted() {
        let booking_id = Uuid::new_v4();
        let rider_id = Uuid::new_v4();
        let listing_id = Uuid::new_v4();
        for rating in 1..=5 {
            assert!(Review::new(booking_id, rider_id, listing_id, rating, None).is_ok());
        }
    }

    #[test]
    fn rating_out_of_range_is_rejected() {
        let booking_id = Uuid::new_v4();
        let rider_id = Uuid::new_v4();
        let listing_id = Uuid::new_v4();
        assert!(Review::new(booking_id, rider_id, listing_id, 0, None).is_err());
        assert!(Review::new(booking_id, rider_id, listing_id, 6, None).is_err());
        assert!(Review::new(booking_id, rider_id, listing_id, -1, None).is_err());
    }
}
