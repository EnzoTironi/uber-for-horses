use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{haversine_km, AnimalKind, Listing, TimeSlot};
use crate::error::AppError;
use crate::repo::ListingRepo;

pub struct NearbyListing {
    pub listing: Listing,
    pub distance_km: f64,
}

pub struct ListingService {
    repo: Arc<dyn ListingRepo>,
}

impl ListingService {
    pub fn new(repo: Arc<dyn ListingRepo>) -> Self {
        Self { repo }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_listing(
        &self,
        owner_id: Uuid,
        kind: AnimalKind,
        name: String,
        description: String,
        photo_url: String,
        hourly_price_cents: i64,
        lat: f64,
        lng: f64,
    ) -> Result<Listing, AppError> {
        if name.trim().is_empty() {
            return Err(AppError::Validation("name must not be empty".into()));
        }
        if hourly_price_cents <= 0 {
            return Err(AppError::Validation(
                "hourly_price_cents must be positive".into(),
            ));
        }
        if !(-90.0..=90.0).contains(&lat) {
            return Err(AppError::Validation("lat out of range".into()));
        }
        if !(-180.0..=180.0).contains(&lng) {
            return Err(AppError::Validation("lng out of range".into()));
        }
        let listing = Listing::new(
            owner_id,
            kind,
            name,
            description,
            photo_url,
            hourly_price_cents,
            lat,
            lng,
        );
        self.repo.create(listing).await
    }

    pub async fn get_listing(&self, id: Uuid) -> Result<Listing, AppError> {
        self.repo
            .get(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("listing {id} not found")))
    }

    pub async fn add_time_slot(
        &self,
        listing_id: Uuid,
        start_at: chrono::DateTime<chrono::Utc>,
        end_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<TimeSlot, AppError> {
        // Ensure listing exists first.
        self.get_listing(listing_id).await?;
        if end_at <= start_at {
            return Err(AppError::Validation("end_at must be after start_at".into()));
        }
        let slot = TimeSlot::new(listing_id, start_at, end_at);
        self.repo.add_time_slot(slot).await
    }

    pub async fn list_time_slots(&self, listing_id: Uuid) -> Result<Vec<TimeSlot>, AppError> {
        self.repo.list_time_slots(listing_id).await
    }

    pub async fn get_time_slot(&self, id: Uuid) -> Result<TimeSlot, AppError> {
        self.repo
            .get_time_slot(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("time slot {id} not found")))
    }

    /// Search active listings within radius_km of (lat, lng), sorted ascending by distance.
    pub async fn search_nearby(
        &self,
        lat: f64,
        lng: f64,
        radius_km: f64,
    ) -> Result<Vec<NearbyListing>, AppError> {
        let listings = self.repo.list_active().await?;
        let mut results: Vec<NearbyListing> = listings
            .into_iter()
            .filter_map(|listing| {
                let distance_km = haversine_km(lat, lng, listing.lat, listing.lng);
                if distance_km <= radius_km {
                    Some(NearbyListing {
                        listing,
                        distance_km,
                    })
                } else {
                    None
                }
            })
            .collect();
        results.sort_by(|a, b| {
            a.distance_km
                .partial_cmp(&b.distance_km)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(results)
    }
}
