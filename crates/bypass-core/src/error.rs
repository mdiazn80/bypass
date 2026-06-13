use thiserror::Error;

/// Unified error type for all `bypass-core` operations.
#[derive(Debug, Error)]
pub enum BypassError {
    /// The secret backend (OS keychain) could not be reached.
    #[error("credential backend unavailable: {0}")]
    BackendUnavailable(String),

    /// The requested context does not exist.
    #[error("context '{0}' not found")]
    ContextNotFound(String),

    /// The requested key does not exist within the given context.
    #[error("key '{0}' not found in context '{1}'")]
    KeyNotFound(String, String),

    /// A context with the same name already exists.
    #[error("context '{0}' already exists")]
    ContextExists(String),

    /// Wraps errors coming from the OS keychain.
    #[error("keychain error: {0}")]
    Keychain(String),

    /// Encryption or decryption failed (wrong key, tampered data, etc.).
    #[error("cryptography error: {0}")]
    Crypto(String),

    /// Filesystem error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// (De)serialization error.
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Data on disk is malformed or fails an invariant.
    #[error("invalid data: {0}")]
    Invalid(String),
}
