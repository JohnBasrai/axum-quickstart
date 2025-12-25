//! WebAuthn configuration and builder.
//!
//! This module provides a factory function for creating a WebAuthn instance
//! configured for the application's relying party identity.

use std::str::FromStr;

use crate::config::WebAuthnConfig;
use anyhow::Result;
use reqwest::Url;
use webauthn_rs::{Webauthn, WebauthnBuilder};

/// Creates a configured WebAuthn instance from application config.
///
/// # Parameters
/// - `config`: WebAuthn configuration (RP ID, origin, etc.)
///
/// # Returns
/// A configured `Webauthn` instance ready for registration/authentication flows.
///
/// # Errors
/// Returns an error if the WebAuthn builder fails to construct a valid instance.
/// This typically happens if the origin URL or RP ID are malformed.
pub fn create_webauthn(config: &WebAuthnConfig) -> Result<Webauthn> {
    // ---
    tracing::debug!("Creating with config:{:?}", config);

    let url = Url::from_str(config.origin.as_str())?;
    let builder = WebauthnBuilder::new(&config.rp_id, &url)?;
    let webauthn = builder.rp_name(&config.rp_name).build()?;

    Ok(webauthn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_webauthn_success() {
        let config = WebAuthnConfig {
            rp_id: "localhost".to_string(),
            rp_name: "Test App".to_string(),
            origin: "http://localhost:8080".to_string(),
        };

        let result = create_webauthn(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn create_webauthn_invalid_origin() {
        let config = WebAuthnConfig {
            rp_id: "localhost".to_string(),
            rp_name: "Test App".to_string(),
            origin: "not-a-valid-url".to_string(),
        };

        let result = create_webauthn(&config);
        assert!(result.is_err());
    }
}
