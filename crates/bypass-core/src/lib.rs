//! Shared core for Bypass.
//!
//! Provides an encrypted credential vault (OS keychain master key +
//! ChaCha20-Poly1305 on-disk store) consumed by the Tauri GUI.

mod backend;
mod crypto;
mod error;
mod keystore;
mod model;
mod vault;

pub use backend::{HybridBackend, SecretBackend};
pub use error::BypassError;
pub use model::CredentialContext;
pub use vault::Vault;

/// Convenience alias used across the crate.
pub type Result<T> = std::result::Result<T, BypassError>;
