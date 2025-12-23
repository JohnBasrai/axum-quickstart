use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a user in the WebAuthn system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    // ---
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    // ---
    pub fn new(username: String) -> Self {
        // ---
        Self {
            id: Uuid::new_v4(),
            username,
            created_at: Utc::now(),
        }
    }
}

/// Represents a WebAuthn credential (passkey) for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    // ---
    /// Unique credential ID (from authenticator)
    pub id: Vec<u8>,
    
    /// User this credential belongs to
    pub user_id: Uuid,
    
    /// Public key for signature verification
    pub public_key: Vec<u8>,
    
    /// Signature counter (for replay attack prevention)
    pub counter: i32,
    
    /// When this credential was created
    pub created_at: DateTime<Utc>,
}

impl Credential {
    // ---
    pub fn new(id: Vec<u8>, user_id: Uuid, public_key: Vec<u8>, counter: i32) -> Self {
        // ---
        Self {
            id,
            user_id,
            public_key,
            counter,
            created_at: Utc::now(),
        }
    }
}
