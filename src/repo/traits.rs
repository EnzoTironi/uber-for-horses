use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{Booking, Listing, Owner, Rider, TimeSlot};
use crate::error::AppError;

#[async_trait]
pub trait OwnerRepo: Send + Sync {
    async fn create(&self, owner: Owner) -> Result<Owner, AppError>;
    async fn get(&self, id: Uuid) -> Result<Option<Owner>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Owner>, AppError>;
}

#[async_trait]
pub trait RiderRepo: Send + Sync {
    async fn create(&self, rider: Rider) -> Result<Rider, AppError>;
    async fn get(&self, id: Uuid) -> Result<Option<Rider>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Rider>, AppError>;
}

#[async_trait]
pub trait ListingRepo: Send + Sync {
    async fn create(&self, listing: Listing) -> Result<Listing, AppError>;
    async fn get(&self, id: Uuid) -> Result<Option<Listing>, AppError>;
    async fn list_active(&self) -> Result<Vec<Listing>, AppError>;

    async fn add_time_slot(&self, slot: TimeSlot) -> Result<TimeSlot, AppError>;
    async fn get_time_slot(&self, id: Uuid) -> Result<Option<TimeSlot>, AppError>;
    async fn list_time_slots(&self, listing_id: Uuid) -> Result<Vec<TimeSlot>, AppError>;
    async fn update_time_slot(&self, slot: TimeSlot) -> Result<TimeSlot, AppError>;
}

#[async_trait]
pub trait BookingRepo: Send + Sync {
    async fn create(&self, booking: Booking) -> Result<Booking, AppError>;
    async fn get(&self, id: Uuid) -> Result<Option<Booking>, AppError>;
    async fn update(&self, booking: Booking) -> Result<Booking, AppError>;
    async fn list_by_rider(&self, rider_id: Uuid) -> Result<Vec<Booking>, AppError>;
    async fn list_by_owner(&self, owner_id: Uuid) -> Result<Vec<Booking>, AppError>;
}
