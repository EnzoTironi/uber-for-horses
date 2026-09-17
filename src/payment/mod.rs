pub mod mock;
pub mod stripe;

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::AppError;

/// Abstraction over a payment processor's authorize/capture/release
/// (escrow-style) flow. `authorize` places a hold on funds and returns an
/// opaque `hold_id`; `capture` finalizes (charges) a held amount; `release`
/// voids/cancels a hold (or refunds a capture) without charging the rider.
#[async_trait]
pub trait PaymentGateway: Send + Sync {
    /// Place an authorization hold for `amount_cents` against the given rider.
    /// Returns an opaque hold id used for subsequent capture/release calls.
    async fn authorize(&self, amount_cents: i64, rider_id: Uuid) -> Result<String, AppError>;

    /// Capture (charge) a previously authorized hold. Fails if the hold is
    /// unknown or has already been captured.
    async fn capture(&self, hold_id: &str) -> Result<(), AppError>;

    /// Release (void, or refund if already captured) a previously authorized
    /// hold. Fails if the hold is unknown or has already been released.
    async fn release(&self, hold_id: &str) -> Result<(), AppError>;
}

pub use mock::{MockPaymentGateway, PaymentCall};
pub use stripe::StripePaymentGateway;
