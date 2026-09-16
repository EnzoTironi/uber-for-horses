use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{Booking, BookingStatus};
use crate::error::AppError;
use crate::repo::{BookingRepo, ListingRepo};

pub struct BookingService {
    booking_repo: Arc<dyn BookingRepo>,
    listing_repo: Arc<dyn ListingRepo>,
}

impl BookingService {
    pub fn new(booking_repo: Arc<dyn BookingRepo>, listing_repo: Arc<dyn ListingRepo>) -> Self {
        Self {
            booking_repo,
            listing_repo,
        }
    }

    pub async fn get_booking(&self, id: Uuid) -> Result<Booking, AppError> {
        self.booking_repo
            .get(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("booking {id} not found")))
    }

    pub async fn request_booking(
        &self,
        listing_id: Uuid,
        rider_id: Uuid,
        time_slot_id: Uuid,
        message_from_rider: Option<String>,
    ) -> Result<Booking, AppError> {
        let listing = self
            .listing_repo
            .get(listing_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("listing {listing_id} not found")))?;

        let slot = self
            .listing_repo
            .get_time_slot(time_slot_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("time slot {time_slot_id} not found")))?;

        if slot.listing_id != listing_id {
            return Err(AppError::Validation(
                "time slot does not belong to the given listing".into(),
            ));
        }

        if slot.is_booked {
            return Err(AppError::Conflict("time slot is already booked".into()));
        }

        let hours = slot.duration_hours_ceil();
        let total_price_cents = listing.hourly_price_cents * hours;

        let booking = Booking::new(
            listing_id,
            rider_id,
            time_slot_id,
            total_price_cents,
            message_from_rider,
        );

        self.booking_repo.create(booking).await
    }

    async fn owner_id_for_booking(&self, booking: &Booking) -> Result<Uuid, AppError> {
        let listing = self
            .listing_repo
            .get(booking.listing_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("listing {} not found", booking.listing_id))
            })?;
        Ok(listing.owner_id)
    }

    pub async fn confirm_booking(
        &self,
        booking_id: Uuid,
        actor_id: Uuid,
    ) -> Result<Booking, AppError> {
        let mut booking = self.get_booking(booking_id).await?;
        let owner_id = self.owner_id_for_booking(&booking).await?;
        if owner_id != actor_id {
            return Err(AppError::Forbidden(
                "only the listing owner can confirm this booking".into(),
            ));
        }
        if !booking.status.can_transition_to(BookingStatus::Confirmed) {
            return Err(AppError::Conflict(format!(
                "booking cannot transition from {:?} to Confirmed",
                booking.status
            )));
        }

        let mut slot = self
            .listing_repo
            .get_time_slot(booking.time_slot_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("time slot {} not found", booking.time_slot_id))
            })?;
        if slot.is_booked {
            return Err(AppError::Conflict("time slot is already booked".into()));
        }
        slot.is_booked = true;
        self.listing_repo.update_time_slot(slot).await?;

        booking.status = BookingStatus::Confirmed;
        booking.updated_at = chrono::Utc::now();
        self.booking_repo.update(booking).await
    }

    pub async fn decline_booking(
        &self,
        booking_id: Uuid,
        actor_id: Uuid,
    ) -> Result<Booking, AppError> {
        let mut booking = self.get_booking(booking_id).await?;
        let owner_id = self.owner_id_for_booking(&booking).await?;
        if owner_id != actor_id {
            return Err(AppError::Forbidden(
                "only the listing owner can decline this booking".into(),
            ));
        }
        if !booking.status.can_transition_to(BookingStatus::Declined) {
            return Err(AppError::Conflict(format!(
                "booking cannot transition from {:?} to Declined",
                booking.status
            )));
        }
        booking.status = BookingStatus::Declined;
        booking.updated_at = chrono::Utc::now();
        self.booking_repo.update(booking).await
    }

    pub async fn cancel_booking(
        &self,
        booking_id: Uuid,
        actor_id: Uuid,
    ) -> Result<Booking, AppError> {
        let mut booking = self.get_booking(booking_id).await?;
        let owner_id = self.owner_id_for_booking(&booking).await?;

        if actor_id != owner_id && actor_id != booking.rider_id {
            return Err(AppError::Forbidden(
                "only the rider or the listing owner can cancel this booking".into(),
            ));
        }

        if !booking.status.can_transition_to(BookingStatus::Cancelled) {
            return Err(AppError::Conflict(format!(
                "booking cannot transition from {:?} to Cancelled",
                booking.status
            )));
        }

        let was_confirmed = booking.status == BookingStatus::Confirmed;

        booking.status = BookingStatus::Cancelled;
        booking.updated_at = chrono::Utc::now();
        let updated = self.booking_repo.update(booking).await?;

        if was_confirmed {
            if let Some(mut slot) = self
                .listing_repo
                .get_time_slot(updated.time_slot_id)
                .await?
            {
                slot.is_booked = false;
                self.listing_repo.update_time_slot(slot).await?;
            }
        }

        Ok(updated)
    }

    pub async fn complete_booking(
        &self,
        booking_id: Uuid,
        actor_id: Uuid,
    ) -> Result<Booking, AppError> {
        let mut booking = self.get_booking(booking_id).await?;
        let owner_id = self.owner_id_for_booking(&booking).await?;
        if owner_id != actor_id {
            return Err(AppError::Forbidden(
                "only the listing owner can complete this booking".into(),
            ));
        }
        if !booking.status.can_transition_to(BookingStatus::Completed) {
            return Err(AppError::Conflict(format!(
                "booking cannot transition from {:?} to Completed",
                booking.status
            )));
        }
        booking.status = BookingStatus::Completed;
        booking.updated_at = chrono::Utc::now();
        self.booking_repo.update(booking).await
    }

    pub async fn list_by_rider(&self, rider_id: Uuid) -> Result<Vec<Booking>, AppError> {
        self.booking_repo.list_by_rider(rider_id).await
    }

    /// List all bookings for listings belonging to the given owner_id.
    ///
    /// The SQLite repo's `list_by_owner` already performs a real SQL join against
    /// `listings.owner_id`, so its results are correct as-is. The in-memory repo has
    /// no notion of ownership and simply returns every booking, so we defensively
    /// re-filter here using the listing repo (source of truth for ownership) — this
    /// makes the method correct for either backend.
    pub async fn list_by_owner(&self, owner_id: Uuid) -> Result<Vec<Booking>, AppError> {
        let owner_listing_ids: std::collections::HashSet<Uuid> = self
            .listing_repo
            .list_active()
            .await?
            .into_iter()
            .filter(|l| l.owner_id == owner_id)
            .map(|l| l.id)
            .collect();

        let candidate = self.booking_repo.list_by_owner(owner_id).await?;
        Ok(candidate
            .into_iter()
            .filter(|b| owner_listing_ids.contains(&b.listing_id))
            .collect())
    }
}
