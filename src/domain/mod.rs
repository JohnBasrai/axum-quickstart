mod metrics;
mod repository;
mod webauthn_models;

// Publicly expose the Metrics abstraction
pub use metrics::{Metrics, MetricsPtr};

// Publicly expose WebAuthn abstractions
pub use repository::{Repository, RepositoryPtr};
pub use webauthn_models::{Credential, User};

pub async fn init_database_with_retry_from_env() -> anyhow::Result<()> {
    // ---
    crate::infrastructure::init_database_with_retry_from_env().await
}
