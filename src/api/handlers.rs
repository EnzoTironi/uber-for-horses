use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::AnimalKind;
use crate::error::AppError;
use crate::service::{BookingService, ListingService, OwnerService, RiderService};

#[derive(Clone)]
pub struct AppState {
    pub owner_service: Arc<OwnerService>,
    pub rider_service: Arc<RiderService>,
    pub listing_service: Arc<ListingService>,
    pub booking_service: Arc<BookingService>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/owners", post(create_owner))
        .route("/owners/:id", get(get_owner))
        .route("/riders", post(create_rider))
        .route("/riders/:id", get(get_rider))
        .route("/listings", post(create_listing))
        .route("/listings/:id", get(get_listing))
        .route("/listings/search", get(search_listings))
        .route(
            "/listings/:id/slots",
            post(create_time_slot).get(list_time_slots),
        )
        .route("/bookings", post(create_booking))
        .route("/bookings/:id", get(get_booking))
        .route("/bookings/:id/confirm", post(confirm_booking))
        .route("/bookings/:id/decline", post(decline_booking))
        .route("/bookings/:id/cancel", post(cancel_booking))
        .route("/bookings/:id/complete", post(complete_booking))
        .route("/riders/:rider_id/bookings", get(list_rider_bookings))
        .route("/owners/:owner_id/bookings", get(list_owner_bookings))
        .with_state(state)
}

// ---------- Owners ----------

#[derive(Deserialize)]
struct CreateOwnerRequest {
    name: String,
    email: String,
}

async fn create_owner(
    State(state): State<AppState>,
    Json(req): Json<CreateOwnerRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let owner = state
        .owner_service
        .create_owner(req.name, req.email)
        .await?;
    Ok(Json(serde_json::to_value(owner).unwrap()))
}

async fn get_owner(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let owner = state.owner_service.get_owner(id).await?;
    Ok(Json(serde_json::to_value(owner).unwrap()))
}

// ---------- Riders ----------

#[derive(Deserialize)]
struct CreateRiderRequest {
    name: String,
    email: String,
}

async fn create_rider(
    State(state): State<AppState>,
    Json(req): Json<CreateRiderRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rider = state
        .rider_service
        .create_rider(req.name, req.email)
        .await?;
    Ok(Json(serde_json::to_value(rider).unwrap()))
}

async fn get_rider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rider = state.rider_service.get_rider(id).await?;
    Ok(Json(serde_json::to_value(rider).unwrap()))
}

// ---------- Listings ----------

#[derive(Deserialize)]
struct CreateListingRequest {
    owner_id: Uuid,
    kind: AnimalKind,
    name: String,
    description: String,
    photo_url: String,
    hourly_price_cents: i64,
    lat: f64,
    lng: f64,
}

async fn create_listing(
    State(state): State<AppState>,
    Json(req): Json<CreateListingRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let listing = state
        .listing_service
        .create_listing(
            req.owner_id,
            req.kind,
            req.name,
            req.description,
            req.photo_url,
            req.hourly_price_cents,
            req.lat,
            req.lng,
        )
        .await?;
    Ok(Json(serde_json::to_value(listing).unwrap()))
}

async fn get_listing(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let listing = state.listing_service.get_listing(id).await?;
    Ok(Json(serde_json::to_value(listing).unwrap()))
}

#[derive(Deserialize)]
struct SearchQuery {
    lat: f64,
    lng: f64,
    radius_km: f64,
}

#[derive(Serialize)]
struct NearbyListingResponse {
    listing: crate::domain::Listing,
    distance_km: f64,
}

async fn search_listings(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let results = state
        .listing_service
        .search_nearby(q.lat, q.lng, q.radius_km)
        .await?;
    let response: Vec<NearbyListingResponse> = results
        .into_iter()
        .map(|nl| NearbyListingResponse {
            listing: nl.listing,
            distance_km: nl.distance_km,
        })
        .collect();
    Ok(Json(serde_json::to_value(response).unwrap()))
}

// ---------- Time slots ----------

#[derive(Deserialize)]
struct CreateTimeSlotRequest {
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
}

async fn create_time_slot(
    State(state): State<AppState>,
    Path(listing_id): Path<Uuid>,
    Json(req): Json<CreateTimeSlotRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let slot = state
        .listing_service
        .add_time_slot(listing_id, req.start_at, req.end_at)
        .await?;
    Ok(Json(serde_json::to_value(slot).unwrap()))
}

async fn list_time_slots(
    State(state): State<AppState>,
    Path(listing_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let slots = state.listing_service.list_time_slots(listing_id).await?;
    Ok(Json(serde_json::to_value(slots).unwrap()))
}

// ---------- Bookings ----------

#[derive(Deserialize)]
struct CreateBookingRequest {
    listing_id: Uuid,
    rider_id: Uuid,
    time_slot_id: Uuid,
    message_from_rider: Option<String>,
}

async fn create_booking(
    State(state): State<AppState>,
    Json(req): Json<CreateBookingRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let booking = state
        .booking_service
        .request_booking(
            req.listing_id,
            req.rider_id,
            req.time_slot_id,
            req.message_from_rider,
        )
        .await?;
    Ok(Json(serde_json::to_value(booking).unwrap()))
}

async fn get_booking(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let booking = state.booking_service.get_booking(id).await?;
    Ok(Json(serde_json::to_value(booking).unwrap()))
}

#[derive(Deserialize)]
struct ActorQuery {
    actor_id: Uuid,
}

async fn confirm_booking(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<ActorQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let booking = state
        .booking_service
        .confirm_booking(id, q.actor_id)
        .await?;
    Ok(Json(serde_json::to_value(booking).unwrap()))
}

async fn decline_booking(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<ActorQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let booking = state
        .booking_service
        .decline_booking(id, q.actor_id)
        .await?;
    Ok(Json(serde_json::to_value(booking).unwrap()))
}

async fn cancel_booking(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<ActorQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let booking = state.booking_service.cancel_booking(id, q.actor_id).await?;
    Ok(Json(serde_json::to_value(booking).unwrap()))
}

async fn complete_booking(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<ActorQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let booking = state
        .booking_service
        .complete_booking(id, q.actor_id)
        .await?;
    Ok(Json(serde_json::to_value(booking).unwrap()))
}

async fn list_rider_bookings(
    State(state): State<AppState>,
    Path(rider_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let bookings = state.booking_service.list_by_rider(rider_id).await?;
    Ok(Json(serde_json::to_value(bookings).unwrap()))
}

async fn list_owner_bookings(
    State(state): State<AppState>,
    Path(owner_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let bookings = state.booking_service.list_by_owner(owner_id).await?;
    Ok(Json(serde_json::to_value(bookings).unwrap()))
}
