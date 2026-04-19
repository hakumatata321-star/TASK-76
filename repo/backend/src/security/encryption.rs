use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

pub fn derive_key(key_material: &str) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(key_material.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

pub fn encrypt_field(plaintext: &str, key_material: &str) -> Result<String, String> {
    let key = derive_key(key_material);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &combined,
    ))
}

pub fn decrypt_field(encrypted: &str, key_material: &str) -> Result<String, String> {
    let key = derive_key(key_material);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let combined = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        encrypted,
    )
    .map_err(|e| e.to_string())?;

    if combined.len() < 12 {
        return Err("Invalid ciphertext".to_string());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| e.to_string())?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}

/// Encrypt raw bytes (for backup files). Returns nonce || ciphertext.
pub fn encrypt_bytes(plaintext: &[u8], key_material: &str) -> Result<Vec<u8>, String> {
    let key = derive_key(key_material);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| e.to_string())?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    Ok(combined)
}

/// Decrypt raw bytes (for backup files). Expects nonce || ciphertext.
pub fn decrypt_bytes(encrypted: &[u8], key_material: &str) -> Result<Vec<u8>, String> {
    let key = derive_key(key_material);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    if encrypted.len() < 12 {
        return Err("Encrypted data too short".to_string());
    }

    let (nonce_bytes, ciphertext) = encrypted.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = "my-secret-encryption-key-32bytes";
        let plaintext = "1HGCM82633A123456";
        let encrypted = encrypt_field(plaintext, key).unwrap();
        let decrypted = decrypt_field(&encrypted, key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_ciphertext_not_plaintext() {
        let key = "my-secret-encryption-key-32bytes";
        let plaintext = "1HGCM82633A123456";
        let encrypted = encrypt_field(plaintext, key).unwrap();
        assert_ne!(encrypted, plaintext);
        assert!(!encrypted.contains(plaintext));
    }

    #[test]
    fn test_wrong_key_fails() {
        let encrypted = encrypt_field("secret", "key1-32-bytes-long-padding-here").unwrap();
        let result = decrypt_field(&encrypted, "key2-32-bytes-long-padding-here");
        assert!(result.is_err());
    }

    #[test]
    fn test_different_encryptions_differ() {
        let key = "test-key-for-encryption-testing!";
        let e1 = encrypt_field("same", key).unwrap();
        let e2 = encrypt_field("same", key).unwrap();
        assert_ne!(e1, e2); // random nonce ensures different ciphertexts
    }
}
