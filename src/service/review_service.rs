use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{BookingStatus, Review};
use crate::error::AppError;
use crate::repo::{BookingRepo, ReviewRepo};

pub struct ReviewService {
    review_repo: Arc<dyn ReviewRepo>,
    booking_repo: Arc<dyn BookingRepo>,
}

impl ReviewService {
    pub fn new(review_repo: Arc<dyn ReviewRepo>, booking_repo: Arc<dyn BookingRepo>) -> Self {
        Self {
            review_repo,
            booking_repo,
        }
    }

    pub async fn create_review(
        &self,
        booking_id: Uuid,
        rider_id: Uuid,
        rating: i32,
        comment: Option<String>,
    ) -> Result<Review, AppError> {
        let booking = self
            .booking_repo
            .get(booking_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("booking {booking_id} not found")))?;

        if booking.rider_id != rider_id {
            return Err(AppError::Forbidden(
                "only the rider who made this booking can review it".into(),
            ));
        }

        if booking.status != BookingStatus::Completed {
            return Err(AppError::Conflict(
                "only completed bookings can be reviewed".into(),
            ));
        }

        if self.review_repo.get_by_booking(booking_id).await?.is_some() {
            return Err(AppError::Conflict(
                "this booking has already been reviewed".into(),
            ));
        }

        let review = Review::new(booking_id, rider_id, booking.listing_id, rating, comment)?;
        self.review_repo.create(review).await
    }

    pub async fn list_for_listing(&self, listing_id: Uuid) -> Result<Vec<Review>, AppError> {
        self.review_repo.list_for_listing(listing_id).await
    }

    /// Compute the average rating and review count for a listing.
    pub async fn rating_summary_for_listing(
        &self,
        listing_id: Uuid,
    ) -> Result<(Option<f64>, i64), AppError> {
        let reviews = self.review_repo.list_for_listing(listing_id).await?;
        let count = reviews.len() as i64;
        if count == 0 {
            return Ok((None, 0));
        }
        let sum: i64 = reviews.iter().map(|r| r.rating as i64).sum();
        let average = sum as f64 / count as f64;
        Ok((Some(average), count))
    }
}
