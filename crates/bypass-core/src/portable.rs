use argon2::Argon2;
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroizing;

use crate::crypto::{self, KEY_LEN};
use crate::error::BypassError;
use crate::model::VaultData;

/// Magic header identifying a Bypass passphrase-encrypted export (v1).
const MAGIC: &[u8; 8] = b"BYPASSV1";
const SALT_LEN: usize = 16;

/// Derives a 32-byte key from a passphrase using Argon2id.
fn derive_key(passphrase: &str, salt: &[u8]) -> Result<Zeroizing<[u8; KEY_LEN]>, BypassError> {
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    Argon2::default()
        .hash_password_into(passphrase.as_bytes(), salt, &mut *key)
        .map_err(|e| BypassError::Crypto(e.to_string()))?;
    Ok(key)
}

/// Serializes and encrypts the vault for migration between machines.
///
/// Layout: `MAGIC(8) || salt(16) || nonce(12) || ciphertext+tag`. The master
/// key from the OS keychain is never used here, so the file can be decrypted
/// on another machine with only the passphrase.
pub(crate) fn export(data: &VaultData, passphrase: &str) -> Result<Vec<u8>, BypassError> {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let key = derive_key(passphrase, &salt)?;
    let plaintext = Zeroizing::new(serde_json::to_vec(data)?);
    let sealed = crypto::seal(&key, &plaintext)?;

    let mut out = Vec::with_capacity(MAGIC.len() + SALT_LEN + sealed.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&sealed);
    Ok(out)
}

/// Decrypts a blob produced by [`export`].
pub(crate) fn import(blob: &[u8], passphrase: &str) -> Result<VaultData, BypassError> {
    if blob.len() < MAGIC.len() + SALT_LEN || &blob[..MAGIC.len()] != MAGIC {
        return Err(BypassError::Invalid("not a Bypass export file".into()));
    }
    let salt = &blob[MAGIC.len()..MAGIC.len() + SALT_LEN];
    let sealed = &blob[MAGIC.len() + SALT_LEN..];
    let key = derive_key(passphrase, salt)?;
    let plaintext = crypto::open(&key, sealed)?;
    let data = serde_json::from_slice(&plaintext)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ContextEntry;

    #[test]
    fn export_import_roundtrip() {
        let mut data = VaultData::default();
        data.contexts.insert(
            "dev".to_string(),
            ContextEntry {
                description: "Development".into(),
                vars: [("TOKEN".to_string(), "abc".to_string())].into_iter().collect(),
                ..Default::default()
            },
        );

        let blob = export(&data, "correct horse battery staple").unwrap();
        assert!(!blob.windows(3).any(|w| w == b"abc"));

        let restored = import(&blob, "correct horse battery staple").unwrap();
        assert_eq!(restored.contexts["dev"].vars["TOKEN"], "abc");

        assert!(import(&blob, "wrong passphrase").is_err());
    }
}
