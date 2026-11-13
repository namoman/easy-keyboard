//! Phase 5 IPC — profiles, clipboard wheel, `.ekb` / clean JSON.

use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::context::window_watcher::{self, ForegroundInfo, ProfileSwitchedPayload};
use crate::snippet::bundle;
use crate::snippet::clipboard_ring::{self, WheelView};
use crate::snippet::persist;
use crate::snippet::profiles::{
    ensure_default_profile, sync_active_slots, switch_profile, AppProfile, MatchRules, ProfileRule,
    DEFAULT_PROFILE_ID,
};
use crate::snippet::SnippetStore;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileStatus {
    pub active_profile_id: String,
    pub pin_locked: bool,
    pub profiles: Vec<ProfileRule>,
    pub foreground: ForegroundInfo,
}

#[tauri::command]
pub fn list_profiles(state: State<'_, AppState>) -> Result<ProfileStatus, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    ensure_default_profile(&mut store);
    Ok(ProfileStatus {
        active_profile_id: store.active_profile_id.clone(),
        pin_locked: store.pin_locked,
        profiles: store.profiles.iter().map(|p| p.rule.clone()).collect(),
        foreground: window_watcher::read_foreground(),
    })
}

#[tauri::command]
pub fn set_profile_pin(
    app: AppHandle,
    state: State<'_, AppState>,
    locked: bool,
) -> Result<bool, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    store.pin_locked = locked;
    drop(store);
    persist_quiet(&app, &*state);
    let _ = app.emit("app://profile-pin", locked);
    Ok(locked)
}

#[tauri::command]
pub fn switch_app_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<ProfileSwitchedPayload, String> {
    // Manual switch ignores pin (user intent).
    let payload = {
        let mut store = state.store.lock().map_err(|e| e.to_string())?;
        let was_pinned = store.pin_locked;
        store.pin_locked = false;
        let p = switch_profile(&mut store, &profile_id)?;
        let payload = ProfileSwitchedPayload {
            profile_id: p.rule.profile_id.clone(),
            name: p.rule.name.clone(),
            exe: String::new(),
            title: String::new(),
        };
        store.pin_locked = was_pinned;
        payload
    };
    persist_quiet(&app, &*state);
    let _ = crate::shortcuts::refresh_all(&app);
    let _ = app.emit("app://profile-switched", &payload);
    Ok(payload)
}

#[tauri::command]
pub fn upsert_profile_rule(
    app: AppHandle,
    state: State<'_, AppState>,
    rule: ProfileRule,
) -> Result<ProfileRule, String> {
    if rule.profile_id.trim().is_empty() {
        return Err("profile_id required".into());
    }
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    ensure_default_profile(&mut store);
    sync_active_slots(&mut store);
    if let Some(existing) = store
        .profiles
        .iter_mut()
        .find(|p| p.rule.profile_id == rule.profile_id)
    {
        existing.rule = rule.clone();
    } else {
        let slots = store.slots.clone();
        store.profiles.push(AppProfile {
            rule: rule.clone(),
            slots,
        });
    }
    drop(store);
    persist_quiet(&app, &*state);
    Ok(rule)
}

#[tauri::command]
pub fn delete_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<(), String> {
    if profile_id == DEFAULT_PROFILE_ID {
        return Err("cannot delete default profile".into());
    }
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    store.profiles.retain(|p| p.rule.profile_id != profile_id);
    if store.active_profile_id == profile_id {
        let _ = switch_profile(&mut store, DEFAULT_PROFILE_ID);
    }
    drop(store);
    persist_quiet(&app, &*state);
    Ok(())
}

#[tauri::command]
pub fn clipboard_wheel_snapshot() -> Result<WheelView, String> {
    Ok(clipboard_ring::snapshot())
}

#[tauri::command]
pub fn clipboard_wheel_prev(app: AppHandle) -> Result<WheelView, String> {
    Ok(clipboard_ring::rotate_prev(&app))
}

#[tauri::command]
pub fn clipboard_wheel_next(app: AppHandle) -> Result<WheelView, String> {
    Ok(clipboard_ring::rotate_next(&app))
}

#[tauri::command]
pub fn clipboard_wheel_paste(app: AppHandle) -> Result<WheelView, String> {
    clipboard_ring::paste_selected(&app)
}

#[tauri::command]
pub fn export_ekb(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> Result<String, String> {
    let store = snapshot_store(&*state)?;
    let bytes = bundle::export_ekb(&store, &password)?;
    let path = PathBuf::from(path);
    bundle::write_bytes(&path, &bytes)?;
    let _ = &app;
    Ok(path.display().to_string())
}

#[tauri::command]
pub fn import_ekb(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> Result<usize, String> {
    let bytes = bundle::read_bytes(PathBuf::from(&path).as_path())?;
    let store = bundle::import_ekb(&bytes, &password)?;
    apply_store(&app, &*state, store)
}

#[tauri::command]
pub fn export_clean_json(
    state: State<'_, AppState>,
    path: String,
) -> Result<String, String> {
    let store = snapshot_store(&*state)?;
    let json = bundle::export_clean_json(&store)?;
    let path = PathBuf::from(path);
    bundle::write_bytes(&path, json.as_bytes())?;
    Ok(path.display().to_string())
}

#[tauri::command]
pub fn import_clean_json(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<usize, String> {
    let raw = bundle::read_bytes(PathBuf::from(&path).as_path())?;
    let text = String::from_utf8(raw).map_err(|e| format!("utf8: {e}"))?;
    let store = bundle::import_clean_json(&text)?;
    apply_store(&app, &*state, store)
}

fn snapshot_store(state: &AppState) -> Result<SnippetStore, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    ensure_default_profile(&mut store);
    sync_active_slots(&mut store);
    Ok(store.clone())
}

fn apply_store(app: &AppHandle, state: &AppState, mut store: SnippetStore) -> Result<usize, String> {
    ensure_default_profile(&mut store);
    let n = store.slots.len();
    {
        let mut guard = state.store.lock().map_err(|e| e.to_string())?;
        *guard = store;
    }
    persist_quiet(app, state);
    crate::shortcuts::refresh_all(app)?;
    let _ = app.emit(
        "app://profile-switched",
        ProfileSwitchedPayload {
            profile_id: DEFAULT_PROFILE_ID.into(),
            name: "imported".into(),
            exe: String::new(),
            title: String::new(),
        },
    );
    Ok(n)
}

fn persist_quiet(app: &AppHandle, state: &AppState) {
    if let Ok(store) = state.store.lock() {
        let _ = persist::save(app, &store);
    }
}

#[allow(dead_code)]
fn _match_rules_export(_: MatchRules) {}
