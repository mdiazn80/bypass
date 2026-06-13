use base64::Engine;
use keyring::Entry;
use zeroize::Zeroizing;

use super::crypto::{self, KEY_LEN};
use super::error::BypassError;

const SERVICE: &str = "com.mdiazn80.bypass";
const ACCOUNT: &str = "master-key";

/// Loads the master key from the OS keychain, creating and storing a fresh
/// random key on first use.
///
/// Returns [`BypassError::BackendUnavailable`] when the keychain itself cannot
/// be reached (e.g. Linux without a running Secret Service), so callers can
/// fall back to a passphrase-derived key.
pub fn load_or_create_master_key() -> Result<Zeroizing<[u8; KEY_LEN]>, BypassError> {
    let entry = Entry::new(SERVICE, ACCOUNT)
        .map_err(|e| BypassError::BackendUnavailable(e.to_string()))?;

    match entry.get_password() {
        Ok(encoded) => decode_key(&encoded),
        Err(keyring::Error::NoEntry) => {
            let key = crypto::random_key();
            let encoded = base64::engine::general_purpose::STANDARD.encode(&*key);
            entry
                .set_password(&encoded)
                .map_err(|e| BypassError::Keychain(e.to_string()))?;
            Ok(key)
        }
        Err(e) => Err(BypassError::BackendUnavailable(e.to_string())),
    }
}

/// Reads a master key from the `BYPASS_MASTER_KEY` environment variable
/// (base64-encoded 32 bytes). Used as a fallback when the OS keychain is not
/// available, e.g. on headless Linux without a Secret Service.
pub fn master_key_from_env() -> Option<Result<Zeroizing<[u8; KEY_LEN]>, BypassError>> {
    match std::env::var("BYPASS_MASTER_KEY") {
        Ok(value) if !value.trim().is_empty() => Some(decode_key(&value)),
        _ => None,
    }
}

fn decode_key(encoded: &str) -> Result<Zeroizing<[u8; KEY_LEN]>, BypassError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded.trim())
        .map_err(|e| BypassError::Crypto(e.to_string()))?;
    if bytes.len() != KEY_LEN {
        return Err(BypassError::Invalid("master key has wrong length".into()));
    }
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    key.copy_from_slice(&bytes);
    Ok(key)
}
