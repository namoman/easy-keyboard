//! `.ekb` personal backup (Argon2id + AES-256-GCM) and clean JSON share.

use std::fs;
use std::path::Path;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::Zeroize;

use super::crypto;
use super::store::SnippetStore;

const MAGIC: &[u8; 4] = b"EKB1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

/// Encrypt full store → `.ekb` bytes: magic || salt || nonce || ciphertext.
pub fn export_ekb(store: &SnippetStore, password: &str) -> Result<Vec<u8>, String> {
    if password.is_empty() {
        return Err("password required".into());
    }
    let json = serde_json::to_vec(store).map_err(|e| format!("serialize: {e}"))?;
    let mut salt = [0u8; SALT_LEN];
    crypto_fill(&mut salt)?;
    let mut key = derive_key(password.as_bytes(), &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("cipher: {e}"))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    crypto_fill(&mut nonce_bytes)?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, json.as_ref())
        .map_err(|e| format!("encrypt: {e}"))?;
    key.zeroize();
    let mut out = Vec::with_capacity(4 + SALT_LEN + NONCE_LEN + ct.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

pub fn import_ekb(blob: &[u8], password: &str) -> Result<SnippetStore, String> {
    if password.is_empty() {
        return Err("password required".into());
    }
    if blob.len() < 4 + SALT_LEN + NONCE_LEN + 16 {
        return Err("ekb too short".into());
    }
    if &blob[..4] != MAGIC {
        return Err("not an EKB1 file".into());
    }
    let salt = &blob[4..4 + SALT_LEN];
    let nonce_bytes = &blob[4 + SALT_LEN..4 + SALT_LEN + NONCE_LEN];
    let ct = &blob[4 + SALT_LEN + NONCE_LEN..];
    let mut key = derive_key(password.as_bytes(), salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("cipher: {e}"))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher
        .decrypt(nonce, ct)
        .map_err(|_| "decrypt failed (wrong password?)".to_string());
    key.zeroize();
    let plain = plain?;
    serde_json::from_slice(&plain).map_err(|e| format!("deserialize: {e}"))
}

/// Public share: Secret slot bodies emptied.
pub fn export_clean_json(store: &SnippetStore) -> Result<String, String> {
    let mut clean = store.clone();
    for slot in &mut clean.slots {
        if slot.secret {
            slot.body.clear();
        }
    }
    for profile in &mut clean.profiles {
        for slot in &mut profile.slots {
            if slot.secret {
                slot.body.clear();
            }
        }
    }
    serde_json::to_string_pretty(&clean).map_err(|e| format!("json: {e}"))
}

pub fn import_clean_json(json: &str) -> Result<SnippetStore, String> {
    serde_json::from_str(json).map_err(|e| format!("json: {e}"))
}

pub fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    fs::write(path, bytes).map_err(|e| format!("write: {e}"))
}

pub fn read_bytes(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("read: {e}"))
}

fn derive_key(password: &[u8], salt: &[u8]) -> Result<[u8; KEY_LEN], String> {
    let params = Params::new(19_456, 2, 1, Some(KEY_LEN)).map_err(|e| format!("argon2 params: {e}"))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; KEY_LEN];
    argon
        .hash_password_into(password, salt, &mut key)
        .map_err(|e| format!("argon2: {e}"))?;
    Ok(key)
}

fn crypto_fill(buf: &mut [u8]) -> Result<(), String> {
    crypto::fill_random(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ekb_roundtrip_and_clean_secrets() {
        let mut store = SnippetStore::default();
        store.slots[0].body = "secret-body".into();
        store.slots[0].secret = true;
        store.slots[1].body = "public".into();
        let blob = export_ekb(&store, "test-pass").expect("export");
        let back = import_ekb(&blob, "test-pass").expect("import");
        assert_eq!(back.slots[0].body, "secret-body");
        let clean = export_clean_json(&store).expect("clean");
        assert!(!clean.contains("secret-body"));
        assert!(clean.contains("public"));
    }
}
