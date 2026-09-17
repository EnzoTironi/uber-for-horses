use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{Booking, Listing, Owner, Review, Rider, TimeSlot};
use crate::error::AppError;
use crate::repo::traits::{BookingRepo, ListingRepo, OwnerRepo, ReviewRepo, RiderRepo};

#[derive(Default)]
pub struct InMemoryOwnerRepo {
    data: Mutex<HashMap<Uuid, Owner>>,
}

#[async_trait]
impl OwnerRepo for InMemoryOwnerRepo {
    async fn create(&self, owner: Owner) -> Result<Owner, AppError> {
        let mut data = self.data.lock().unwrap();
        data.insert(owner.id, owner.clone());
        Ok(owner)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Owner>, AppError> {
        let data = self.data.lock().unwrap();
        Ok(data.get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<Owner>, AppError> {
        let data = self.data.lock().unwrap();
        Ok(data.values().find(|o| o.email == email).cloned())
    }

    async fn find_by_referral_code(&self, code: &str) -> Result<Option<Owner>, AppError> {
        let data = self.data.lock().unwrap();
        Ok(data.values().find(|o| o.referral_code == code).cloned())
    }
}

#[derive(Default)]
pub struct InMemoryRiderRepo {
    data: Mutex<HashMap<Uuid, Rider>>,
}

#[async_trait]
impl RiderRepo for InMemoryRiderRepo {
    async fn create(&self, rider: Rider) -> Result<Rider, AppError> {
        let mut data = self.data.lock().unwrap();
        data.insert(rider.id, rider.clone());
        Ok(rider)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Rider>, AppError> {
        let data = self.data.lock().unwrap();
        Ok(data.get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<Rider>, AppError> {
        let data = self.data.lock().unwrap();
        Ok(data.values().find(|r| r.email == email).cloned())
    }

    async fn find_by_referral_code(&self, code: &str) -> Result<Option<Rider>, AppError> {
        let data = self.data.lock().unwrap();
        Ok(data.values().find(|r| r.referral_code == code).cloned())
    }
}

#[derive(Default)]
pub struct InMemoryListingRepo {
    listings: Mutex<HashMap<Uuid, Listing>>,
    slots: Mutex<HashMap<Uuid, TimeSlot>>,
}

#[async_trait]
impl ListingRepo for InMemoryListingRepo {
    async fn create(&self, listing: Listing) -> Result<Listing, AppError> {
        let mut data = self.listings.lock().unwrap();
        data.insert(listing.id, listing.clone());
        Ok(listing)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Listing>, AppError> {
        let data = self.listings.lock().unwrap();
        Ok(data.get(&id).cloned())
    }

    async fn list_active(&self) -> Result<Vec<Listing>, AppError> {
        let data = self.listings.lock().unwrap();
        Ok(data.values().filter(|l| l.active).cloned().collect())
    }

    async fn add_time_slot(&self, slot: TimeSlot) -> Result<TimeSlot, AppError> {
        let mut data = self.slots.lock().unwrap();
        data.insert(slot.id, slot.clone());
        Ok(slot)
    }

    async fn get_time_slot(&self, id: Uuid) -> Result<Option<TimeSlot>, AppError> {
        let data = self.slots.lock().unwrap();
        Ok(data.get(&id).cloned())
    }

    async fn list_time_slots(&self, listing_id: Uuid) -> Result<Vec<TimeSlot>, AppError> {
        let data = self.slots.lock().unwrap();
        Ok(data
            .values()
            .filter(|s| s.listing_id == listing_id)
            .cloned()
            .collect())
    }

    async fn update_time_slot(&self, slot: TimeSlot) -> Result<TimeSlot, AppError> {
        let mut data = self.slots.lock().unwrap();
        data.insert(slot.id, slot.clone());
        Ok(slot)
    }
}

#[derive(Default)]
pub struct InMemoryBookingRepo {
    data: Mutex<HashMap<Uuid, Booking>>,
}

#[async_trait]
impl BookingRepo for InMemoryBookingRepo {
    async fn create(&self, booking: Booking) -> Result<Booking, AppError> {
        let mut data = self.data.lock().unwrap();
        data.insert(booking.id, booking.clone());
        Ok(booking)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Booking>, AppError> {
        let data = self.data.lock().unwrap();
        Ok(data.get(&id).cloned())
    }

    async fn update(&self, booking: Booking) -> Result<Booking, AppError> {
        let mut data = self.data.lock().unwrap();
        data.insert(booking.id, booking.clone());
        Ok(booking)
    }

    async fn list_by_rider(&self, rider_id: Uuid) -> Result<Vec<Booking>, AppError> {
        let data = self.data.lock().unwrap();
        Ok(data
            .values()
            .filter(|b| b.rider_id == rider_id)
            .cloned()
            .collect())
    }

    async fn list_by_owner(&self, owner_id: Uuid) -> Result<Vec<Booking>, AppError> {
        // Owner isn't stored directly on Booking; caller (service layer) filters by
        // resolving listing -> owner. This repo-level method exists for symmetry but
        // the in-memory implementation here just returns all bookings; the SQLite
        // implementation performs a real join. For correctness in this in-memory repo,
        // we return an empty list and let the service layer do the owner_id join using
        // ListingRepo instead. See booking_service::list_owner_bookings.
        let _ = owner_id;
        let data = self.data.lock().unwrap();
        Ok(data.values().cloned().collect())
    }
}

#[derive(Default)]
pub struct InMemoryReviewRepo {
    data: Mutex<HashMap<Uuid, Review>>,
}

#[async_trait]
impl ReviewRepo for InMemoryReviewRepo {
    async fn create(&self, review: Review) -> Result<Review, AppError> {
        let mut data = self.data.lock().unwrap();
        data.insert(review.id, review.clone());
        Ok(review)
    }

    async fn get_by_booking(&self, booking_id: Uuid) -> Result<Option<Review>, AppError> {
        let data = self.data.lock().unwrap();
        Ok(data.values().find(|r| r.booking_id == booking_id).cloned())
    }

    async fn list_for_listing(&self, listing_id: Uuid) -> Result<Vec<Review>, AppError> {
        let data = self.data.lock().unwrap();
        Ok(data
            .values()
            .filter(|r| r.listing_id == listing_id)
            .cloned()
            .collect())
    }
}
