use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroizing;

use super::error::BypassError;

/// Length of the symmetric master key in bytes.
pub const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

/// Generates a fresh random 32-byte key using the OS CSPRNG.
pub fn random_key() -> Zeroizing<[u8; KEY_LEN]> {
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    OsRng.fill_bytes(&mut *key);
    key
}

/// Encrypts `plaintext` with ChaCha20-Poly1305.
///
/// The returned buffer is `nonce (12 bytes) || ciphertext+tag`. A fresh random
/// nonce is generated for every call.
pub fn seal(key: &[u8; KEY_LEN], plaintext: &[u8]) -> Result<Vec<u8>, BypassError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| BypassError::Crypto(e.to_string()))?;

    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypts a buffer produced by [`seal`]. The plaintext is wrapped in
/// `Zeroizing` so it is wiped from memory when dropped.
pub fn open(key: &[u8; KEY_LEN], data: &[u8]) -> Result<Zeroizing<Vec<u8>>, BypassError> {
    if data.len() < NONCE_LEN {
        return Err(BypassError::Invalid("ciphertext too short".into()));
    }
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|e| BypassError::Crypto(e.to_string()))?;
    Ok(Zeroizing::new(plaintext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_open_roundtrip() {
        let key = random_key();
        let msg = b"super secret value";
        let sealed = seal(&key, msg).unwrap();
        assert_ne!(&sealed[12..], msg);
        let opened = open(&key, &sealed).unwrap();
        assert_eq!(&*opened, msg);
    }

    #[test]
    fn wrong_key_fails() {
        let key = random_key();
        let other = random_key();
        let sealed = seal(&key, b"data").unwrap();
        assert!(open(&other, &sealed).is_err());
    }

    #[test]
    fn nonce_is_random() {
        let key = random_key();
        let a = seal(&key, b"data").unwrap();
        let b = seal(&key, b"data").unwrap();
        assert_ne!(a, b);
    }
}
