use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{Booking, BookingStatus};
use crate::error::AppError;
use crate::payment::PaymentGateway;
use crate::repo::{BookingRepo, ListingRepo};

pub struct BookingService {
    booking_repo: Arc<dyn BookingRepo>,
    listing_repo: Arc<dyn ListingRepo>,
    payment_gateway: Arc<dyn PaymentGateway>,
}

impl BookingService {
    pub fn new(
        booking_repo: Arc<dyn BookingRepo>,
        listing_repo: Arc<dyn ListingRepo>,
        payment_gateway: Arc<dyn PaymentGateway>,
    ) -> Self {
        Self {
            booking_repo,
            listing_repo,
            payment_gateway,
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

        let hold_id = self
            .payment_gateway
            .authorize(total_price_cents, rider_id)
            .await?;

        let mut booking = Booking::new(
            listing_id,
            rider_id,
            time_slot_id,
            total_price_cents,
            message_from_rider,
        );
        booking.payment_hold_id = Some(hold_id);

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

        let hold_id = booking.payment_hold_id.clone().ok_or_else(|| {
            AppError::Internal(format!(
                "booking {booking_id} has no payment hold to capture"
            ))
        })?;
        self.payment_gateway.capture(&hold_id).await?;

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
        if let Some(hold_id) = booking.payment_hold_id.clone() {
            self.payment_gateway.release(&hold_id).await?;
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
        let hold_id = booking.payment_hold_id.clone();

        booking.status = BookingStatus::Cancelled;
        booking.updated_at = chrono::Utc::now();
        let updated = self.booking_repo.update(booking).await?;

        if let Some(hold_id) = hold_id {
            self.payment_gateway.release(&hold_id).await?;
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AnimalKind, Listing, Owner, Rider, TimeSlot};
    use crate::payment::{MockPaymentGateway, PaymentCall};
    use crate::repo::in_memory::{InMemoryBookingRepo, InMemoryListingRepo};

    struct Fixture {
        service: BookingService,
        gateway: Arc<MockPaymentGateway>,
        listing_id: Uuid,
        time_slot_id: Uuid,
        rider_id: Uuid,
        owner_id: Uuid,
        hourly_price_cents: i64,
    }

    async fn setup() -> Fixture {
        let booking_repo = Arc::new(InMemoryBookingRepo::default());
        let listing_repo = Arc::new(InMemoryListingRepo::default());
        let gateway = Arc::new(MockPaymentGateway::new());

        let owner = Owner::new(
            "Jane Owner".into(),
            "jane@example.com".into(),
            "hash".into(),
            None,
        );
        let owner_id = owner.id;

        let rider = Rider::new(
            "Rick Rider".into(),
            "rick@example.com".into(),
            "hash".into(),
            None,
        );
        let rider_id = rider.id;

        let hourly_price_cents = 2_000;
        let listing = Listing::new(
            owner_id,
            AnimalKind::Horse,
            "Trusty".into(),
            "A good horse".into(),
            "http://example.com/trusty.jpg".into(),
            hourly_price_cents,
            0.0,
            0.0,
        );
        let listing_id = listing.id;
        listing_repo.create(listing).await.unwrap();

        let start = chrono::Utc::now();
        let end = start + chrono::Duration::hours(2);
        let slot = TimeSlot::new(listing_id, start, end);
        let time_slot_id = slot.id;
        listing_repo.add_time_slot(slot).await.unwrap();

        let service = BookingService::new(booking_repo, listing_repo, gateway.clone());

        Fixture {
            service,
            gateway,
            listing_id,
            time_slot_id,
            rider_id,
            owner_id,
            hourly_price_cents,
        }
    }

    #[tokio::test]
    async fn request_booking_authorizes_correct_amount_and_stores_hold_id() {
        let f = setup().await;
        let booking = f
            .service
            .request_booking(f.listing_id, f.rider_id, f.time_slot_id, None)
            .await
            .unwrap();

        // 2 hour slot at 2000 cents/hr = 4000 cents.
        let expected_amount = f.hourly_price_cents * 2;
        assert_eq!(booking.total_price_cents, expected_amount);
        assert!(booking.payment_hold_id.is_some());

        let calls = f.gateway.calls();
        assert_eq!(
            calls,
            vec![PaymentCall::Authorize {
                amount_cents: expected_amount,
                rider_id: f.rider_id,
            }]
        );
    }

    #[tokio::test]
    async fn confirm_booking_captures_hold_exactly_once_and_transitions_to_confirmed() {
        let f = setup().await;
        let booking = f
            .service
            .request_booking(f.listing_id, f.rider_id, f.time_slot_id, None)
            .await
            .unwrap();
        let hold_id = booking.payment_hold_id.clone().unwrap();

        let confirmed = f
            .service
            .confirm_booking(booking.id, f.owner_id)
            .await
            .unwrap();

        assert_eq!(confirmed.status, BookingStatus::Confirmed);

        let captures = f
            .gateway
            .calls()
            .into_iter()
            .filter(|c| matches!(c, PaymentCall::Capture { .. }))
            .count();
        assert_eq!(captures, 1);
        assert!(f
            .gateway
            .calls()
            .contains(&PaymentCall::Capture { hold_id }));
    }

    #[tokio::test]
    async fn confirm_booking_does_not_transition_when_capture_fails() {
        let f = setup().await;
        let booking = f
            .service
            .request_booking(f.listing_id, f.rider_id, f.time_slot_id, None)
            .await
            .unwrap();
        let hold_id = booking.payment_hold_id.clone().unwrap();

        // Pre-capture the hold out-of-band so the service's capture call fails
        // (simulating a declined charge) and confirm_booking should surface the
        // error without transitioning the booking to Confirmed.
        f.gateway.capture(&hold_id).await.unwrap();

        let err = f.service.confirm_booking(booking.id, f.owner_id).await;
        assert!(err.is_err());

        let still_requested = f.service.get_booking(booking.id).await.unwrap();
        assert_eq!(still_requested.status, BookingStatus::Requested);
    }

    #[tokio::test]
    async fn decline_booking_releases_the_hold() {
        let f = setup().await;
        let booking = f
            .service
            .request_booking(f.listing_id, f.rider_id, f.time_slot_id, None)
            .await
            .unwrap();
        let hold_id = booking.payment_hold_id.clone().unwrap();

        let declined = f
            .service
            .decline_booking(booking.id, f.owner_id)
            .await
            .unwrap();
        assert_eq!(declined.status, BookingStatus::Declined);

        assert!(f.gateway.calls().contains(&PaymentCall::Release {
            hold_id: hold_id.clone()
        }));
        let releases = f
            .gateway
            .calls()
            .into_iter()
            .filter(|c| matches!(c, PaymentCall::Release { .. }))
            .count();
        assert_eq!(releases, 1);
    }

    #[tokio::test]
    async fn cancel_confirmed_booking_releases_the_captured_hold_refund_path() {
        let f = setup().await;
        let booking = f
            .service
            .request_booking(f.listing_id, f.rider_id, f.time_slot_id, None)
            .await
            .unwrap();
        let hold_id = booking.payment_hold_id.clone().unwrap();

        f.service
            .confirm_booking(booking.id, f.owner_id)
            .await
            .unwrap();

        let cancelled = f
            .service
            .cancel_booking(booking.id, f.owner_id)
            .await
            .unwrap();
        assert_eq!(cancelled.status, BookingStatus::Cancelled);

        assert!(f
            .gateway
            .calls()
            .contains(&PaymentCall::Release { hold_id }));
    }

    #[tokio::test]
    async fn capture_is_never_called_twice_on_the_same_hold() {
        let f = setup().await;
        let booking = f
            .service
            .request_booking(f.listing_id, f.rider_id, f.time_slot_id, None)
            .await
            .unwrap();

        f.service
            .confirm_booking(booking.id, f.owner_id)
            .await
            .unwrap();

        // A second confirm attempt is rejected by the state machine before it
        // ever reaches the gateway (Confirmed -> Confirmed is not a legal
        // transition), so capture must still only have been called once.
        let second = f.service.confirm_booking(booking.id, f.owner_id).await;
        assert!(second.is_err());

        let captures = f
            .gateway
            .calls()
            .into_iter()
            .filter(|c| matches!(c, PaymentCall::Capture { .. }))
            .count();
        assert_eq!(captures, 1);
    }

    #[tokio::test]
    async fn release_is_never_called_twice_on_the_same_hold() {
        let f = setup().await;
        let booking = f
            .service
            .request_booking(f.listing_id, f.rider_id, f.time_slot_id, None)
            .await
            .unwrap();

        f.service
            .decline_booking(booking.id, f.owner_id)
            .await
            .unwrap();

        // A second decline attempt is rejected by the state machine before it
        // ever reaches the gateway (Declined -> Declined is not a legal
        // transition), so release must still only have been called once.
        let second = f.service.decline_booking(booking.id, f.owner_id).await;
        assert!(second.is_err());

        let releases = f
            .gateway
            .calls()
            .into_iter()
            .filter(|c| matches!(c, PaymentCall::Release { .. }))
            .count();
        assert_eq!(releases, 1);
    }
}
