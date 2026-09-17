use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::jwt::{self, AuthIdentity};
use crate::domain::{AnimalKind, Role};
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
        .route("/auth/signup", post(signup))
        .route("/auth/login", post(login))
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

// ---------- Auth ----------

#[derive(Deserialize)]
struct SignupRequest {
    name: String,
    email: String,
    password: String,
    role: Role,
    #[serde(default)]
    referred_by: Option<String>,
}

#[derive(Serialize)]
struct TokenResponse {
    token: String,
    referral_code: String,
}

async fn signup(
    State(state): State<AppState>,
    Json(req): Json<SignupRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    // If a referral code was supplied, resolve it to an existing account's
    // code. An unknown/garbage code is not an error — it is simply ignored,
    // and the new account is created without attribution.
    let referred_by = if let Some(code) = req.referred_by.as_deref() {
        let code = code.trim();
        if code.is_empty() {
            None
        } else {
            let owner_match = state.owner_service.find_by_referral_code(code).await?;
            if owner_match.is_some() {
                Some(code.to_string())
            } else {
                let rider_match = state.rider_service.find_by_referral_code(code).await?;
                rider_match.map(|_| code.to_string())
            }
        }
    } else {
        None
    };

    let (id, role, referral_code) = match req.role {
        Role::Owner => {
            let owner = state
                .owner_service
                .create_owner(req.name, req.email, req.password, referred_by)
                .await?;
            (owner.id, Role::Owner, owner.referral_code)
        }
        Role::Rider => {
            let rider = state
                .rider_service
                .create_rider(req.name, req.email, req.password, referred_by)
                .await?;
            (rider.id, Role::Rider, rider.referral_code)
        }
    };

    let token = jwt::issue_token(id, role)?;
    Ok(Json(TokenResponse {
        token,
        referral_code,
    }))
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
    role: Role,
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let (id, password_hash, role, referral_code) = match req.role {
        Role::Owner => {
            let owner = state
                .owner_service
                .find_by_email(&req.email)
                .await?
                .ok_or_else(|| AppError::Unauthorized("invalid email or password".into()))?;
            (
                owner.id,
                owner.password_hash,
                Role::Owner,
                owner.referral_code,
            )
        }
        Role::Rider => {
            let rider = state
                .rider_service
                .find_by_email(&req.email)
                .await?
                .ok_or_else(|| AppError::Unauthorized("invalid email or password".into()))?;
            (
                rider.id,
                rider.password_hash,
                Role::Rider,
                rider.referral_code,
            )
        }
    };

    let valid = bcrypt::verify(&req.password, &password_hash)
        .map_err(|e| AppError::Internal(format!("failed to verify password: {e}")))?;
    if !valid {
        return Err(AppError::Unauthorized("invalid email or password".into()));
    }

    let token = jwt::issue_token(id, role)?;
    Ok(Json(TokenResponse {
        token,
        referral_code,
    }))
}

// ---------- Owners ----------

#[derive(Deserialize)]
struct CreateOwnerRequest {
    name: String,
    email: String,
    password: String,
}

async fn create_owner(
    State(state): State<AppState>,
    Json(req): Json<CreateOwnerRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let owner = state
        .owner_service
        .create_owner(req.name, req.email, req.password, None)
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
    password: String,
}

async fn create_rider(
    State(state): State<AppState>,
    Json(req): Json<CreateRiderRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rider = state
        .rider_service
        .create_rider(req.name, req.email, req.password, None)
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
    identity: AuthIdentity,
    Json(req): Json<CreateListingRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if identity.role != Role::Owner {
        return Err(AppError::Forbidden(
            "only owners can create listings".into(),
        ));
    }
    // owner_id is derived from the verified token, never trusted from the
    // request body — otherwise any authenticated owner could create a
    // listing "owned" by someone else's id.
    let listing = state
        .listing_service
        .create_listing(
            identity.id,
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
    identity: AuthIdentity,
    Json(req): Json<CreateTimeSlotRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if identity.role != Role::Owner {
        return Err(AppError::Forbidden("only owners can add time slots".into()));
    }
    let listing = state.listing_service.get_listing(listing_id).await?;
    if listing.owner_id != identity.id {
        return Err(AppError::Forbidden(
            "only the listing owner can add time slots to this listing".into(),
        ));
    }
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
    time_slot_id: Uuid,
    message_from_rider: Option<String>,
}

async fn create_booking(
    State(state): State<AppState>,
    identity: AuthIdentity,
    Json(req): Json<CreateBookingRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if identity.role != Role::Rider {
        return Err(AppError::Forbidden(
            "only riders can request bookings".into(),
        ));
    }
    // rider_id is derived from the verified token, never trusted from the
    // request body — otherwise anyone could book (and rack up a bill) on
    // behalf of an arbitrary rider_id, authenticated or not.
    let booking = state
        .booking_service
        .request_booking(
            req.listing_id,
            identity.id,
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

async fn confirm_booking(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    identity: AuthIdentity,
) -> Result<Json<serde_json::Value>, AppError> {
    if identity.role != Role::Owner {
        return Err(AppError::Forbidden(
            "only owners can confirm bookings".into(),
        ));
    }
    let booking = state
        .booking_service
        .confirm_booking(id, identity.id)
        .await?;
    Ok(Json(serde_json::to_value(booking).unwrap()))
}

async fn decline_booking(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    identity: AuthIdentity,
) -> Result<Json<serde_json::Value>, AppError> {
    if identity.role != Role::Owner {
        return Err(AppError::Forbidden(
            "only owners can decline bookings".into(),
        ));
    }
    let booking = state
        .booking_service
        .decline_booking(id, identity.id)
        .await?;
    Ok(Json(serde_json::to_value(booking).unwrap()))
}

async fn cancel_booking(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    identity: AuthIdentity,
) -> Result<Json<serde_json::Value>, AppError> {
    // Both owners and riders can cancel; ownership/rider-match is enforced in
    // the service layer using the verified identity.
    let booking = state
        .booking_service
        .cancel_booking(id, identity.id)
        .await?;
    Ok(Json(serde_json::to_value(booking).unwrap()))
}

async fn complete_booking(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    identity: AuthIdentity,
) -> Result<Json<serde_json::Value>, AppError> {
    if identity.role != Role::Owner {
        return Err(AppError::Forbidden(
            "only owners can complete bookings".into(),
        ));
    }
    let booking = state
        .booking_service
        .complete_booking(id, identity.id)
        .await?;
    Ok(Json(serde_json::to_value(booking).unwrap()))
}

async fn list_rider_bookings(
    State(state): State<AppState>,
    Path(rider_id): Path<Uuid>,
    identity: AuthIdentity,
) -> Result<Json<serde_json::Value>, AppError> {
    if identity.id != rider_id {
        return Err(AppError::Forbidden(
            "you can only view your own booking history".into(),
        ));
    }
    let bookings = state.booking_service.list_by_rider(rider_id).await?;
    Ok(Json(serde_json::to_value(bookings).unwrap()))
}

async fn list_owner_bookings(
    State(state): State<AppState>,
    Path(owner_id): Path<Uuid>,
    identity: AuthIdentity,
) -> Result<Json<serde_json::Value>, AppError> {
    if identity.id != owner_id {
        return Err(AppError::Forbidden(
            "you can only view your own booking history".into(),
        ));
    }
    let bookings = state.booking_service.list_by_owner(owner_id).await?;
    Ok(Json(serde_json::to_value(bookings).unwrap()))
}
