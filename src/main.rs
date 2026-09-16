use std::sync::Arc;

use uber_for_horses::api::AppState;
use uber_for_horses::repo::sqlite::{
    init_pool, SqliteBookingRepo, SqliteListingRepo, SqliteOwnerRepo, SqliteRiderRepo,
};
use uber_for_horses::service::{BookingService, ListingService, OwnerService, RiderService};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data.db".to_string());

    tracing::info!("connecting to database at {database_url}");
    let pool = init_pool(&database_url)
        .await
        .expect("failed to initialize database pool");

    let owner_repo = Arc::new(SqliteOwnerRepo::new(pool.clone()));
    let rider_repo = Arc::new(SqliteRiderRepo::new(pool.clone()));
    let listing_repo = Arc::new(SqliteListingRepo::new(pool.clone()));
    let booking_repo = Arc::new(SqliteBookingRepo::new(pool));

    let state = AppState {
        owner_service: Arc::new(OwnerService::new(owner_repo)),
        rider_service: Arc::new(RiderService::new(rider_repo)),
        listing_service: Arc::new(ListingService::new(listing_repo.clone())),
        booking_service: Arc::new(BookingService::new(booking_repo, listing_repo)),
    };

    let app = uber_for_horses::api::router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind to 0.0.0.0:8080");

    tracing::info!("uber_for_horses listening on 0.0.0.0:8080");

    axum::serve(listener, app).await.expect("server error");
}
