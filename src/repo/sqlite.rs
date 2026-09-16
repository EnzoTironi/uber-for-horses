use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::{AnimalKind, Booking, BookingStatus, Listing, Owner, Rider, TimeSlot};
use crate::error::AppError;
use crate::repo::traits::{BookingRepo, ListingRepo, OwnerRepo, RiderRepo};

pub async fn init_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS owners (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS riders (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS listings (
            id TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            photo_url TEXT NOT NULL,
            hourly_price_cents INTEGER NOT NULL,
            lat REAL NOT NULL,
            lng REAL NOT NULL,
            active INTEGER NOT NULL,
            created_at TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS time_slots (
            id TEXT PRIMARY KEY,
            listing_id TEXT NOT NULL,
            start_at TEXT NOT NULL,
            end_at TEXT NOT NULL,
            is_booked INTEGER NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS bookings (
            id TEXT PRIMARY KEY,
            listing_id TEXT NOT NULL,
            rider_id TEXT NOT NULL,
            time_slot_id TEXT NOT NULL,
            status TEXT NOT NULL,
            total_price_cents INTEGER NOT NULL,
            message_from_rider TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

fn kind_to_str(kind: AnimalKind) -> &'static str {
    match kind {
        AnimalKind::Horse => "horse",
        AnimalKind::Carriage => "carriage",
    }
}

fn str_to_kind(s: &str) -> AnimalKind {
    match s {
        "carriage" => AnimalKind::Carriage,
        _ => AnimalKind::Horse,
    }
}

fn status_to_str(status: BookingStatus) -> &'static str {
    match status {
        BookingStatus::Requested => "requested",
        BookingStatus::Confirmed => "confirmed",
        BookingStatus::Declined => "declined",
        BookingStatus::Cancelled => "cancelled",
        BookingStatus::Completed => "completed",
    }
}

fn str_to_status(s: &str) -> BookingStatus {
    match s {
        "confirmed" => BookingStatus::Confirmed,
        "declined" => BookingStatus::Declined,
        "cancelled" => BookingStatus::Cancelled,
        "completed" => BookingStatus::Completed,
        _ => BookingStatus::Requested,
    }
}

pub struct SqliteOwnerRepo {
    pool: SqlitePool,
}

impl SqliteOwnerRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OwnerRepo for SqliteOwnerRepo {
    async fn create(&self, owner: Owner) -> Result<Owner, AppError> {
        sqlx::query("INSERT INTO owners (id, name, email, created_at) VALUES (?, ?, ?, ?)")
            .bind(owner.id.to_string())
            .bind(&owner.name)
            .bind(&owner.email)
            .bind(owner.created_at.to_rfc3339())
            .execute(&self.pool)
            .await?;
        Ok(owner)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Owner>, AppError> {
        let row = sqlx::query("SELECT id, name, email, created_at FROM owners WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Owner {
            id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
            name: r.get("name"),
            email: r.get("email"),
            created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                .unwrap()
                .with_timezone(&Utc),
        }))
    }
}

pub struct SqliteRiderRepo {
    pool: SqlitePool,
}

impl SqliteRiderRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RiderRepo for SqliteRiderRepo {
    async fn create(&self, rider: Rider) -> Result<Rider, AppError> {
        sqlx::query("INSERT INTO riders (id, name, email, created_at) VALUES (?, ?, ?, ?)")
            .bind(rider.id.to_string())
            .bind(&rider.name)
            .bind(&rider.email)
            .bind(rider.created_at.to_rfc3339())
            .execute(&self.pool)
            .await?;
        Ok(rider)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Rider>, AppError> {
        let row = sqlx::query("SELECT id, name, email, created_at FROM riders WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Rider {
            id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
            name: r.get("name"),
            email: r.get("email"),
            created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                .unwrap()
                .with_timezone(&Utc),
        }))
    }
}

pub struct SqliteListingRepo {
    pool: SqlitePool,
}

impl SqliteListingRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn row_to_listing(r: sqlx::sqlite::SqliteRow) -> Listing {
    Listing {
        id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
        owner_id: Uuid::parse_str(r.get::<String, _>("owner_id").as_str()).unwrap(),
        kind: str_to_kind(r.get::<String, _>("kind").as_str()),
        name: r.get("name"),
        description: r.get("description"),
        photo_url: r.get("photo_url"),
        hourly_price_cents: r.get("hourly_price_cents"),
        lat: r.get("lat"),
        lng: r.get("lng"),
        active: r.get::<i64, _>("active") != 0,
        created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
            .unwrap()
            .with_timezone(&Utc),
    }
}

fn row_to_slot(r: sqlx::sqlite::SqliteRow) -> TimeSlot {
    TimeSlot {
        id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
        listing_id: Uuid::parse_str(r.get::<String, _>("listing_id").as_str()).unwrap(),
        start_at: DateTime::parse_from_rfc3339(r.get::<String, _>("start_at").as_str())
            .unwrap()
            .with_timezone(&Utc),
        end_at: DateTime::parse_from_rfc3339(r.get::<String, _>("end_at").as_str())
            .unwrap()
            .with_timezone(&Utc),
        is_booked: r.get::<i64, _>("is_booked") != 0,
    }
}

#[async_trait]
impl ListingRepo for SqliteListingRepo {
    async fn create(&self, listing: Listing) -> Result<Listing, AppError> {
        sqlx::query(
            r#"INSERT INTO listings
            (id, owner_id, kind, name, description, photo_url, hourly_price_cents, lat, lng, active, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(listing.id.to_string())
        .bind(listing.owner_id.to_string())
        .bind(kind_to_str(listing.kind))
        .bind(&listing.name)
        .bind(&listing.description)
        .bind(&listing.photo_url)
        .bind(listing.hourly_price_cents)
        .bind(listing.lat)
        .bind(listing.lng)
        .bind(listing.active as i64)
        .bind(listing.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(listing)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Listing>, AppError> {
        let row = sqlx::query("SELECT * FROM listings WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(row_to_listing))
    }

    async fn list_active(&self) -> Result<Vec<Listing>, AppError> {
        let rows = sqlx::query("SELECT * FROM listings WHERE active = 1")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(row_to_listing).collect())
    }

    async fn add_time_slot(&self, slot: TimeSlot) -> Result<TimeSlot, AppError> {
        sqlx::query(
            "INSERT INTO time_slots (id, listing_id, start_at, end_at, is_booked) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(slot.id.to_string())
        .bind(slot.listing_id.to_string())
        .bind(slot.start_at.to_rfc3339())
        .bind(slot.end_at.to_rfc3339())
        .bind(slot.is_booked as i64)
        .execute(&self.pool)
        .await?;
        Ok(slot)
    }

    async fn get_time_slot(&self, id: Uuid) -> Result<Option<TimeSlot>, AppError> {
        let row = sqlx::query("SELECT * FROM time_slots WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(row_to_slot))
    }

    async fn list_time_slots(&self, listing_id: Uuid) -> Result<Vec<TimeSlot>, AppError> {
        let rows = sqlx::query("SELECT * FROM time_slots WHERE listing_id = ?")
            .bind(listing_id.to_string())
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(row_to_slot).collect())
    }

    async fn update_time_slot(&self, slot: TimeSlot) -> Result<TimeSlot, AppError> {
        sqlx::query("UPDATE time_slots SET is_booked = ? WHERE id = ?")
            .bind(slot.is_booked as i64)
            .bind(slot.id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(slot)
    }
}

pub struct SqliteBookingRepo {
    pool: SqlitePool,
}

impl SqliteBookingRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn row_to_booking(r: sqlx::sqlite::SqliteRow) -> Booking {
    Booking {
        id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
        listing_id: Uuid::parse_str(r.get::<String, _>("listing_id").as_str()).unwrap(),
        rider_id: Uuid::parse_str(r.get::<String, _>("rider_id").as_str()).unwrap(),
        time_slot_id: Uuid::parse_str(r.get::<String, _>("time_slot_id").as_str()).unwrap(),
        status: str_to_status(r.get::<String, _>("status").as_str()),
        total_price_cents: r.get("total_price_cents"),
        message_from_rider: r.get("message_from_rider"),
        created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
            .unwrap()
            .with_timezone(&Utc),
        updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
            .unwrap()
            .with_timezone(&Utc),
    }
}

#[async_trait]
impl BookingRepo for SqliteBookingRepo {
    async fn create(&self, booking: Booking) -> Result<Booking, AppError> {
        sqlx::query(
            r#"INSERT INTO bookings
            (id, listing_id, rider_id, time_slot_id, status, total_price_cents, message_from_rider, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(booking.id.to_string())
        .bind(booking.listing_id.to_string())
        .bind(booking.rider_id.to_string())
        .bind(booking.time_slot_id.to_string())
        .bind(status_to_str(booking.status))
        .bind(booking.total_price_cents)
        .bind(&booking.message_from_rider)
        .bind(booking.created_at.to_rfc3339())
        .bind(booking.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(booking)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Booking>, AppError> {
        let row = sqlx::query("SELECT * FROM bookings WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(row_to_booking))
    }

    async fn update(&self, booking: Booking) -> Result<Booking, AppError> {
        sqlx::query("UPDATE bookings SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status_to_str(booking.status))
            .bind(booking.updated_at.to_rfc3339())
            .bind(booking.id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(booking)
    }

    async fn list_by_rider(&self, rider_id: Uuid) -> Result<Vec<Booking>, AppError> {
        let rows = sqlx::query("SELECT * FROM bookings WHERE rider_id = ?")
            .bind(rider_id.to_string())
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(row_to_booking).collect())
    }

    async fn list_by_owner(&self, owner_id: Uuid) -> Result<Vec<Booking>, AppError> {
        let rows = sqlx::query(
            r#"SELECT b.* FROM bookings b
               JOIN listings l ON b.listing_id = l.id
               WHERE l.owner_id = ?"#,
        )
        .bind(owner_id.to_string())
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_booking).collect())
    }
}
