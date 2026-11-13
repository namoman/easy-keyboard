//! Load / save encrypted `snippets.enc` under the app data directory.

use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use super::crypto::{self, zeroize_key};
use super::store::SnippetStore;

const FILE_NAME: &str = "snippets.enc";

pub fn enc_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))?;
    fs::create_dir_all(&dir).map_err(|e| format!("create app data: {e}"))?;
    Ok(dir.join(FILE_NAME))
}

pub fn save(app: &AppHandle, store: &SnippetStore) -> Result<PathBuf, String> {
    let path = enc_path(app)?;
    let json = serde_json::to_vec(store).map_err(|e| format!("serialize: {e}"))?;
    let mut key = crypto::load_or_create_key()?;
    let blob = crypto::encrypt(&json, &key);
    zeroize_key(&mut key);
    let blob = blob?;
    fs::write(&path, blob).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(path)
}

pub fn load(app: &AppHandle) -> Result<Option<SnippetStore>, String> {
    let path = enc_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let blob = fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut key = crypto::load_or_create_key()?;
    let plain = crypto::decrypt(&blob, &key);
    zeroize_key(&mut key);
    let plain = plain?;
    let store: SnippetStore =
        serde_json::from_slice(&plain).map_err(|e| format!("deserialize: {e}"))?;
    Ok(Some(store))
}
