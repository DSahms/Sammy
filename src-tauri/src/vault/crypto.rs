use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{SammyError, SammyResult};

pub const SALT_LEN: usize = 16;
pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 12;

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct MasterKey(pub [u8; KEY_LEN]);

pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

pub fn derive_key(passphrase: &str, salt: &[u8]) -> SammyResult<MasterKey> {
    // m_cost ~19 MiB, t_cost 2, p_cost 1 — Argon2id
    let params = Params::new(19_456, 2, 1, Some(KEY_LEN))
        .map_err(|e| SammyError::msg(format!("argon2 params: {e}")))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|e| SammyError::msg(format!("argon2: {e}")))?;
    Ok(MasterKey(key))
}

/// SQLCipher passphrase derived from master key (hex).
pub fn sqlcipher_key(master: &MasterKey) -> String {
    hex::encode(master.0)
}

pub fn encrypt_bytes(master: &MasterKey, plaintext: &[u8]) -> SammyResult<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(&master.0)
        .map_err(|e| SammyError::msg(format!("aes key: {e}")))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let mut out = nonce_bytes.to_vec();
    let ct = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| SammyError::msg(format!("encrypt: {e}")))?;
    out.extend(ct);
    Ok(out)
}

#[allow(dead_code)]
pub fn decrypt_bytes(master: &MasterKey, blob: &[u8]) -> SammyResult<Vec<u8>> {
    if blob.len() < NONCE_LEN + 1 {
        return Err(SammyError::msg("ciphertext too short"));
    }
    let cipher = Aes256Gcm::new_from_slice(&master.0)
        .map_err(|e| SammyError::msg(format!("aes key: {e}")))?;
    let (nonce_bytes, ct) = blob.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ct)
        .map_err(|_| SammyError::BadPassphrase)
}
