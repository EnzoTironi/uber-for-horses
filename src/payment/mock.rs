use std::sync::Mutex;

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::AppError;
use crate::payment::PaymentGateway;

/// A single call made against a [`MockPaymentGateway`], recorded so tests can
/// assert exactly what happened (and how many times).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentCall {
    Authorize { amount_cents: i64, rider_id: Uuid },
    Capture { hold_id: String },
    Release { hold_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HoldState {
    Authorized,
    Captured,
    Released,
}

struct Hold {
    state: HoldState,
}

/// In-memory [`PaymentGateway`] for tests. `authorize` always succeeds and
/// returns a fresh hold id; `capture`/`release` enforce that a hold exists
/// and hasn't already been captured/released (no double capture/release).
#[derive(Default)]
pub struct MockPaymentGateway {
    calls: Mutex<Vec<PaymentCall>>,
    holds: Mutex<std::collections::HashMap<String, Hold>>,
}

impl MockPaymentGateway {
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot of every call made against this gateway, in order.
    pub fn calls(&self) -> Vec<PaymentCall> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl PaymentGateway for MockPaymentGateway {
    async fn authorize(&self, amount_cents: i64, rider_id: Uuid) -> Result<String, AppError> {
        let hold_id = format!("hold_{}", Uuid::new_v4());
        self.holds.lock().unwrap().insert(
            hold_id.clone(),
            Hold {
                state: HoldState::Authorized,
            },
        );
        self.calls.lock().unwrap().push(PaymentCall::Authorize {
            amount_cents,
            rider_id,
        });
        Ok(hold_id)
    }

    async fn capture(&self, hold_id: &str) -> Result<(), AppError> {
        self.calls.lock().unwrap().push(PaymentCall::Capture {
            hold_id: hold_id.to_string(),
        });
        let mut holds = self.holds.lock().unwrap();
        let hold = holds
            .get_mut(hold_id)
            .ok_or_else(|| AppError::NotFound(format!("payment hold {hold_id} not found")))?;
        match hold.state {
            HoldState::Authorized => {
                hold.state = HoldState::Captured;
                Ok(())
            }
            HoldState::Captured => Err(AppError::Conflict(format!(
                "payment hold {hold_id} was already captured"
            ))),
            HoldState::Released => Err(AppError::Conflict(format!(
                "payment hold {hold_id} was already released"
            ))),
        }
    }

    async fn release(&self, hold_id: &str) -> Result<(), AppError> {
        self.calls.lock().unwrap().push(PaymentCall::Release {
            hold_id: hold_id.to_string(),
        });
        let mut holds = self.holds.lock().unwrap();
        let hold = holds
            .get_mut(hold_id)
            .ok_or_else(|| AppError::NotFound(format!("payment hold {hold_id} not found")))?;
        match hold.state {
            HoldState::Released => Err(AppError::Conflict(format!(
                "payment hold {hold_id} was already released"
            ))),
            HoldState::Authorized | HoldState::Captured => {
                hold.state = HoldState::Released;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn authorize_returns_fresh_hold_ids() {
        let gw = MockPaymentGateway::new();
        let rider = Uuid::new_v4();
        let a = gw.authorize(1000, rider).await.unwrap();
        let b = gw.authorize(1000, rider).await.unwrap();
        assert_ne!(a, b);
    }

    #[tokio::test]
    async fn capture_unknown_hold_errors() {
        let gw = MockPaymentGateway::new();
        assert!(gw.capture("nope").await.is_err());
    }

    #[tokio::test]
    async fn double_capture_is_conflict() {
        let gw = MockPaymentGateway::new();
        let hold = gw.authorize(1000, Uuid::new_v4()).await.unwrap();
        gw.capture(&hold).await.unwrap();
        let err = gw.capture(&hold).await.unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[tokio::test]
    async fn double_release_is_conflict() {
        let gw = MockPaymentGateway::new();
        let hold = gw.authorize(1000, Uuid::new_v4()).await.unwrap();
        gw.release(&hold).await.unwrap();
        let err = gw.release(&hold).await.unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[tokio::test]
    async fn release_after_capture_is_allowed_refund_path() {
        let gw = MockPaymentGateway::new();
        let hold = gw.authorize(1000, Uuid::new_v4()).await.unwrap();
        gw.capture(&hold).await.unwrap();
        gw.release(&hold).await.unwrap();
    }
}
