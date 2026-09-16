use std::sync::Arc;

use crate::domain::Rider;
use crate::error::AppError;
use crate::repo::RiderRepo;

pub struct RiderService {
    repo: Arc<dyn RiderRepo>,
}

impl RiderService {
    pub fn new(repo: Arc<dyn RiderRepo>) -> Self {
        Self { repo }
    }

    pub async fn create_rider(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<Rider, AppError> {
        if name.trim().is_empty() {
            return Err(AppError::Validation("name must not be empty".into()));
        }
        if email.trim().is_empty() || !email.contains('@') {
            return Err(AppError::Validation("email must be a valid address".into()));
        }
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("failed to hash password: {e}")))?;
        let rider = Rider::new(name, email, password_hash);
        self.repo.create(rider).await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<Rider>, AppError> {
        self.repo.find_by_email(email).await
    }

    pub async fn get_rider(&self, id: uuid::Uuid) -> Result<Rider, AppError> {
        self.repo
            .get(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("rider {id} not found")))
    }
}
