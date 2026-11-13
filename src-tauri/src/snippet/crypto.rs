//! AES-256-GCM at-rest encryption. Key lives in OS keyring (DPAPI on Windows).

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use keyring::Entry;
use zeroize::Zeroize;

const SERVICE: &str = "com.namoman.easy-keyboard";
const ACCOUNT: &str = "snippets-aes-key-v1";
const NONCE_LEN: usize = 12;

/// Load 32-byte key from keyring, or create + store a new one.
pub fn load_or_create_key() -> Result<[u8; 32], String> {
    let entry = Entry::new(SERVICE, ACCOUNT).map_err(|e| format!("keyring entry: {e}"))?;
    match entry.get_password() {
        Ok(secret) => decode_key(&secret),
        Err(keyring::Error::NoEntry) => {
            let key = fresh_key()?;
            let encoded = hex::encode(key);
            entry
                .set_password(&encoded)
                .map_err(|e| format!("keyring set: {e}"))?;
            Ok(key)
        }
        Err(e) => Err(format!("keyring get: {e}")),
    }
}

/// Encrypt plaintext → `nonce (12) || ciphertext+tag`.
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("cipher: {e}"))?;
    let nonce_bytes = fresh_nonce()?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let mut out = nonce_bytes.to_vec();
    let ct = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("encrypt: {e}"))?;
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Decrypt `nonce || ciphertext+tag`.
pub fn decrypt(blob: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if blob.len() < NONCE_LEN + 16 {
        return Err("ciphertext too short".into());
    }
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("cipher: {e}"))?;
    let (nonce_bytes, ct) = blob.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ct)
        .map_err(|e| format!("decrypt: {e}"))
}

fn decode_key(secret: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(secret.trim()).map_err(|e| format!("key hex: {e}"))?;
    if bytes.len() != 32 {
        return Err("key must be 32 bytes".into());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

fn fresh_key() -> Result<[u8; 32], String> {
    let mut key = [0u8; 32];
    fill_random(&mut key)?;
    Ok(key)
}

fn fresh_nonce() -> Result<[u8; NONCE_LEN], String> {
    let mut nonce = [0u8; NONCE_LEN];
    fill_random(&mut nonce)?;
    Ok(nonce)
}

/// Fill buffer with OS CSPRNG bytes (also used by `.ekb` salt/nonce).
pub fn fill_random(buf: &mut [u8]) -> Result<(), String> {
    #[cfg(windows)]
    {
        #[link(name = "bcrypt")]
        extern "system" {
            fn BCryptGenRandom(
                h_algorithm: usize,
                pb_buffer: *mut u8,
                cb_buffer: u32,
                dw_flags: u32,
            ) -> i32;
        }
        const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x00000002;
        let status = unsafe {
            BCryptGenRandom(
                0,
                buf.as_mut_ptr(),
                buf.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };
        if status != 0 {
            return Err(format!("BCryptGenRandom failed: {status}"));
        }
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
            .hash(&mut hasher);
        std::process::id().hash(&mut hasher);
        let mut state = hasher.finish();
        for b in buf.iter_mut() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            *b = (state >> 33) as u8;
        }
        Ok(())
    }
}

/// Zeroize a key buffer when dropping sensitive material at call sites.
pub fn zeroize_key(key: &mut [u8; 32]) {
    key.zeroize();
}

/// Tiny hex codec (avoid extra crate weight for 32-byte keys).
mod hex {
    pub fn encode(bytes: [u8; 32]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(64);
        for b in bytes {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0xf) as usize] as char);
        }
        out
    }

    pub fn decode(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("odd hex length".into());
        }
        let mut out = Vec::with_capacity(s.len() / 2);
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let hi = from_hex(bytes[i])?;
            let lo = from_hex(bytes[i + 1])?;
            out.push((hi << 4) | lo);
            i += 2;
        }
        Ok(out)
    }

    fn from_hex(b: u8) -> Result<u8, String> {
        match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            b'A'..=b'F' => Ok(b - b'A' + 10),
            _ => Err("invalid hex".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let key = [7u8; 32];
        let pt = b"hello secret snippets";
        let blob = encrypt(pt, &key).expect("encrypt");
        let out = decrypt(&blob, &key).expect("decrypt");
        assert_eq!(out, pt);
    }
}
