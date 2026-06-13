//! Encrypted credential vault: an OS-keychain-held master key encrypts an
//! on-disk store (`store.enc`) with ChaCha20-Poly1305.
//!
//! This was previously the standalone `bypass-core` crate, shared with a CLI
//! companion. With the CLI removed, the GUI is the only consumer, so it lives
//! here as a module.

mod backend;
mod crypto;
mod error;
mod keystore;
mod model;
mod vault;

pub use error::BypassError;
pub use model::CredentialContext;
pub use vault::Vault;
