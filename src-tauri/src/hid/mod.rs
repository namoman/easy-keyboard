//! Phase 2 — `514C:8850` HID bridge + dual trigger profiles.

mod k8850;
mod packet;
mod triggers;

pub use k8850::{disconnected_status, DeviceStatus, LayerRecord, WriteVerifyResult, K8850Device};
#[allow(unused_imports)] // public API surface for callers / docs
pub use packet::{PID, VID};
pub use triggers::{trigger_accel, TriggerProfile};

use std::sync::Mutex;

use tauri::State;

pub struct HidState {
    pub device: Mutex<Option<K8850Device>>,
}

impl Default for HidState {
    fn default() -> Self {
        Self {
            device: Mutex::new(None),
        }
    }
}

#[tauri::command]
pub fn hid_connect(state: State<'_, HidState>) -> Result<DeviceStatus, String> {
    let mut guard = state.device.lock().map_err(|_| "hid state lock poisoned")?;
    let device = K8850Device::open()?;
    let status = device.status();
    *guard = Some(device);
    Ok(status)
}

#[tauri::command]
pub fn hid_disconnect(state: State<'_, HidState>) -> Result<DeviceStatus, String> {
    let mut guard = state.device.lock().map_err(|_| "hid state lock poisoned")?;
    *guard = None;
    Ok(disconnected_status())
}

#[tauri::command]
pub fn hid_status(state: State<'_, HidState>) -> Result<DeviceStatus, String> {
    let guard = state.device.lock().map_err(|_| "hid state lock poisoned")?;
    Ok(match guard.as_ref() {
        Some(dev) => dev.status(),
        None => disconnected_status(),
    })
}

#[tauri::command]
pub fn hid_write_trigger(
    state: State<'_, HidState>,
    layer: u8,
    key: u8,
    profile: TriggerProfile,
) -> Result<WriteVerifyResult, String> {
    k8850::validate_key(key)?;
    let mut guard = state.device.lock().map_err(|_| "hid state lock poisoned")?;
    let device = guard.as_mut().ok_or("pad not connected — call hid_connect first")?;
    device.bind_key(layer, key, profile)
}

#[tauri::command]
pub fn hid_verify_trigger(
    state: State<'_, HidState>,
    layer: u8,
    key: u8,
    profile: TriggerProfile,
) -> Result<bool, String> {
    k8850::validate_key(key)?;
    let mut guard = state.device.lock().map_err(|_| "hid state lock poisoned")?;
    let device = guard.as_mut().ok_or("pad not connected — call hid_connect first")?;
    device.verify_write(layer, key, profile)
}

#[tauri::command]
pub fn hid_read_layer(
    state: State<'_, HidState>,
    layer: u8,
) -> Result<Vec<LayerRecord>, String> {
    let mut guard = state.device.lock().map_err(|_| "hid state lock poisoned")?;
    let device = guard.as_mut().ok_or("pad not connected — call hid_connect first")?;
    device.read_layer(layer)
}

#[tauri::command]
pub fn hid_trigger_accel(profile: TriggerProfile, key: u8) -> Result<String, String> {
    trigger_accel(profile, key)
}
