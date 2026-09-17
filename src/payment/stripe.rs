use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::payment::PaymentGateway;

const STRIPE_API_BASE: &str = "https://api.stripe.com/v1";

/// [`PaymentGateway`] backed by Stripe's PaymentIntents API, using a manual
/// capture flow to model authorize/capture/release (escrow-style) holds:
///
/// - `authorize` creates a PaymentIntent with `capture_method=manual`.
/// - `capture` calls `POST /v1/payment_intents/:id/capture`.
/// - `release` calls `POST /v1/payment_intents/:id/cancel`.
///
/// NOTE: this implementation has not been exercised against a live Stripe
/// account in this environment (no `STRIPE_SECRET_KEY` / network access to
/// Stripe here) — it is shaped correctly against Stripe's documented REST
/// API but is untested live. Treat it as a starting point that needs a real
/// integration test / sandbox run before production use.
pub struct StripePaymentGateway {
    client: reqwest::Client,
    secret_key: String,
}

impl StripePaymentGateway {
    pub fn new(secret_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            secret_key,
        }
    }

    /// Build a `StripePaymentGateway` from the `STRIPE_SECRET_KEY` env var.
    pub fn from_env() -> Result<Self, AppError> {
        let secret_key = std::env::var("STRIPE_SECRET_KEY").map_err(|_| {
            AppError::Internal(
                "STRIPE_SECRET_KEY is not set; cannot construct StripePaymentGateway".into(),
            )
        })?;
        Ok(Self::new(secret_key))
    }
}

#[derive(Debug, Deserialize)]
struct PaymentIntentResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct StripeErrorBody {
    error: StripeErrorDetail,
}

#[derive(Debug, Deserialize)]
struct StripeErrorDetail {
    message: String,
}

async fn stripe_error_message(resp: reqwest::Response) -> String {
    let status = resp.status();
    match resp.json::<StripeErrorBody>().await {
        Ok(body) => format!("stripe error ({status}): {}", body.error.message),
        Err(_) => format!("stripe error: HTTP {status}"),
    }
}

#[async_trait]
impl PaymentGateway for StripePaymentGateway {
    async fn authorize(&self, amount_cents: i64, rider_id: Uuid) -> Result<String, AppError> {
        let params = [
            ("amount".to_string(), amount_cents.to_string()),
            ("currency".to_string(), "usd".to_string()),
            ("capture_method".to_string(), "manual".to_string()),
            ("metadata[rider_id]".to_string(), rider_id.to_string()),
        ];

        let resp = self
            .client
            .post(format!("{STRIPE_API_BASE}/payment_intents"))
            .basic_auth(&self.secret_key, Some(""))
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("stripe request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::Internal(stripe_error_message(resp).await));
        }

        let intent: PaymentIntentResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("stripe response parse failed: {e}")))?;

        Ok(intent.id)
    }

    async fn capture(&self, hold_id: &str) -> Result<(), AppError> {
        let resp = self
            .client
            .post(format!(
                "{STRIPE_API_BASE}/payment_intents/{hold_id}/capture"
            ))
            .basic_auth(&self.secret_key, Some(""))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("stripe request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::Internal(stripe_error_message(resp).await));
        }
        Ok(())
    }

    async fn release(&self, hold_id: &str) -> Result<(), AppError> {
        let resp = self
            .client
            .post(format!(
                "{STRIPE_API_BASE}/payment_intents/{hold_id}/cancel"
            ))
            .basic_auth(&self.secret_key, Some(""))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("stripe request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::Internal(stripe_error_message(resp).await));
        }
        Ok(())
    }
}
