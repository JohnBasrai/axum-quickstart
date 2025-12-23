use super::webauthn_models::{Credential, User};
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

/// Abstraction for WebAuthn data persistence.
#[async_trait::async_trait]
pub trait Repository: Send + Sync {
    // ---
    /// Create a new user.
    async fn create_user(&self, username: &str) -> Result<User>;

    /// Get user by username.
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;

    /// Get user by ID.
    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>>;

    /// Save a new credential for a user.
    async fn save_credential(&self, credential: Credential) -> Result<()>;

    /// Get all credentials for a user.
    async fn get_credentials_by_user(&self, user_id: Uuid) -> Result<Vec<Credential>>;

    /// Get a specific credential by its ID.
    async fn get_credential_by_id(&self, credential_id: &[u8]) -> Result<Option<Credential>>;

    /// Update an existing credential (typically to increment counter).
    async fn update_credential(&self, credential: Credential) -> Result<()>;

    /// Delete a credential by its ID.
    async fn delete_credential(&self, credential_id: &[u8]) -> Result<()>;
}

/// Type alias for any backend that implements Repository.
pub type RepositoryPtr = Arc<dyn Repository>;
