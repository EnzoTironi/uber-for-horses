use std::sync::Arc;

use crate::domain::Owner;
use crate::error::AppError;
use crate::repo::OwnerRepo;

pub struct OwnerService {
    repo: Arc<dyn OwnerRepo>,
}

impl OwnerService {
    pub fn new(repo: Arc<dyn OwnerRepo>) -> Self {
        Self { repo }
    }

    pub async fn create_owner(&self, name: String, email: String) -> Result<Owner, AppError> {
        if name.trim().is_empty() {
            return Err(AppError::Validation("name must not be empty".into()));
        }
        if email.trim().is_empty() || !email.contains('@') {
            return Err(AppError::Validation("email must be a valid address".into()));
        }
        let owner = Owner::new(name, email);
        self.repo.create(owner).await
    }

    pub async fn get_owner(&self, id: uuid::Uuid) -> Result<Owner, AppError> {
        self.repo
            .get(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("owner {id} not found")))
    }
}
