// Gateway module - controls public API for handlers
// Modules are private, only exported symbols are public

mod health;
mod metrics;
mod movies;
mod root;
mod shared_types;
mod webauthn_authenticate;
mod webauthn_credentials;
mod webauthn_register;

// Core handlers
pub use health::health_check;
pub use metrics::metrics_handler;
pub use root::root_handler;

// Movie CRUD handlers
pub use movies::{add_movie, delete_movie, get_movie, update_movie};

// WebAuthn registration handlers
pub use webauthn_register::{register_finish, register_start};

// WebAuthn authentication handlers
pub use webauthn_authenticate::{auth_finish, auth_start};

// WebAuthn credential management handlers
pub use webauthn_credentials::{delete_credential, list_credentials};
