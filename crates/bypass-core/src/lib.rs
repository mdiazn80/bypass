//! Shared core for Bypass.
//!
//! Provides an encrypted credential vault (OS keychain master key +
//! ChaCha20-Poly1305 on-disk store) and hierarchical active-context
//! resolution, used by both the Tauri GUI and the CLI companion.

mod backend;
mod crypto;
mod error;
mod keystore;
mod model;
mod portable;
mod resolver;
mod vault;

pub use backend::{HybridBackend, SecretBackend};
pub use error::BypassError;
pub use model::CredentialContext;
pub use resolver::{resolve_active_context, ResolvedContext, ResolutionSource};
pub use vault::Vault;

/// Convenience alias used across the crate.
pub type Result<T> = std::result::Result<T, BypassError>;
