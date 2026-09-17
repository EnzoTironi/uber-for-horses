pub mod api;
pub mod auth;
pub mod domain;
pub mod error;
pub mod payment;
pub mod repo;
pub mod service;

use std::sync::Arc;

use api::AppState;
use payment::MockPaymentGateway;
use repo::in_memory::{
    InMemoryBookingRepo, InMemoryListingRepo, InMemoryOwnerRepo, InMemoryReviewRepo,
    InMemoryRiderRepo,
};
use service::{BookingService, ListingService, OwnerService, ReviewService, RiderService};

/// Build an AppState backed entirely by the in-memory repositories. Handy for tests
/// and for quickly spinning up the API without a database.
pub fn build_in_memory_state() -> AppState {
    let owner_repo = Arc::new(InMemoryOwnerRepo::default());
    let rider_repo = Arc::new(InMemoryRiderRepo::default());
    let listing_repo = Arc::new(InMemoryListingRepo::default());
    let booking_repo = Arc::new(InMemoryBookingRepo::default());
    let review_repo = Arc::new(InMemoryReviewRepo::default());
    let payment_gateway = Arc::new(MockPaymentGateway::new());

    AppState {
        owner_service: Arc::new(OwnerService::new(owner_repo)),
        rider_service: Arc::new(RiderService::new(rider_repo)),
        listing_service: Arc::new(ListingService::new(listing_repo.clone())),
        booking_service: Arc::new(BookingService::new(
            booking_repo.clone(),
            listing_repo,
            payment_gateway,
        )),
        review_service: Arc::new(ReviewService::new(review_repo, booking_repo)),
    }
}
