//! Tauri IPC — snippets, inject routing, OSD, persist.

use tauri::{AppHandle, Manager, State};
use tauri_plugin_global_shortcut::ShortcutState;

use crate::osd::{self, OsdPulsePayload};
use crate::snippet::persist;
use crate::snippet::profiles::{ensure_default_profile, sync_active_slots};
use crate::snippet::{inject, slot_index, InjectConfig, SnippetSlot};
use crate::state::AppState;
use crate::shortcuts;

#[tauri::command]
pub fn list_slots(state: State<'_, AppState>) -> Result<Vec<SnippetSlot>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    Ok(store.slots.clone())
}

#[tauri::command]
pub fn save_snippet(
    app: AppHandle,
    state: State<'_, AppState>,
    slot: SnippetSlot,
) -> Result<SnippetSlot, String> {
    let idx = slot_index(slot.layer, slot.key).ok_or_else(|| "invalid layer/key".to_string())?;
    let saved = {
        let mut store = state.store.lock().map_err(|e| e.to_string())?;
        let target = store
            .slots
            .get_mut(idx)
            .ok_or_else(|| "slot out of range".to_string())?;
        let mut next = slot;
        if next.label.trim().is_empty() && !next.body.is_empty() {
            next.label = auto_label(&next.body);
        }
        *target = next.clone();
        sync_active_slots(&mut store);
        next
    };
    shortcuts::refresh_all(&app)?;
    let _ = persist_store(&app, &*state);
    Ok(saved)
}

#[tauri::command]
pub fn register_trigger(
    app: AppHandle,
    state: State<'_, AppState>,
    layer: u8,
    key: u8,
    trigger: String,
) -> Result<SnippetSlot, String> {
    let idx = slot_index(layer, key).ok_or_else(|| "invalid layer/key".to_string())?;
    let trimmed = trigger.trim().to_string();
    let saved = {
        let mut store = state.store.lock().map_err(|e| e.to_string())?;
        if !trimmed.is_empty() {
            for (i, s) in store.slots.iter_mut().enumerate() {
                if i != idx && s.trigger.eq_ignore_ascii_case(&trimmed) {
                    s.trigger.clear();
                }
            }
        }
        let target = store
            .slots
            .get_mut(idx)
            .ok_or_else(|| "slot out of range".to_string())?;
        target.trigger = trimmed;
        target.clone()
    };
    shortcuts::refresh_all(&app)?;
    let _ = persist_store(&app, &*state);
    Ok(saved)
}

#[tauri::command]
pub fn paste_test(app: AppHandle, state: State<'_, AppState>, layer: u8, key: u8) -> Result<(), String> {
    let slot = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let idx = slot_index(layer, key).ok_or_else(|| "invalid layer/key".to_string())?;
        store
            .slots
            .get(idx)
            .cloned()
            .ok_or_else(|| "slot out of range".to_string())?
    };
    run_inject(&app, &*state, &slot)
}

#[tauri::command]
pub fn get_clipboard_text() -> Result<String, String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("clipboard open: {e}"))?;
    Ok(clipboard.get_text().unwrap_or_default())
}

#[tauri::command]
pub fn set_timing(
    state: State<'_, AppState>,
    restore_delay_ms: Option<u64>,
    modifier_settle_ms: Option<u64>,
    typing_delay_ms: Option<u64>,
) -> Result<(), String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    if let Some(ms) = restore_delay_ms {
        store.set_restore_delay_ms(ms);
    }
    if let Some(ms) = modifier_settle_ms {
        store.set_modifier_settle_ms(ms);
    }
    if let Some(ms) = typing_delay_ms {
        store.set_typing_delay_ms(ms);
    }
    Ok(())
}

#[tauri::command]
pub fn persist_snippets(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    persist_store(&app, &*state)
}

#[tauri::command]
pub fn osd_toggle(app: AppHandle) -> Result<bool, String> {
    osd::toggle_visible(&app)
}

#[tauri::command]
pub fn osd_set_visible(app: AppHandle, visible: bool) -> Result<bool, String> {
    osd::set_visible(&app, visible)
}

#[tauri::command]
pub fn osd_toggle_ghost(app: AppHandle) -> Result<bool, String> {
    osd::toggle_ghost(&app)
}

#[tauri::command]
pub fn osd_status() -> Result<OsdStatus, String> {
    Ok(OsdStatus {
        visible: osd::is_visible(),
        ghost: osd::is_ghost(),
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OsdStatus {
    pub visible: bool,
    pub ghost: bool,
}

/// OSD inline edit → same path as save_snippet.
#[tauri::command]
pub fn osd_save_slot(
    app: AppHandle,
    state: State<'_, AppState>,
    slot: SnippetSlot,
) -> Result<SnippetSlot, String> {
    save_snippet(app, state, slot)
}

pub fn inject_by_trigger(app: &AppHandle, trigger: &str) -> Result<(), String> {
    let state = app.state::<AppState>();
    let slot = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let Some(slot) = store.find_by_trigger(trigger) else {
            return Ok(());
        };
        if slot.body.is_empty() {
            return Ok(());
        }
        slot.clone()
    };
    run_inject(app, &*state, &slot)
}

fn run_inject(app: &AppHandle, state: &AppState, slot: &SnippetSlot) -> Result<(), String> {
    let config = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        InjectConfig {
            modifier_settle_ms: store.modifier_settle_ms,
            restore_delay_ms: store.restore_delay_ms,
            typing_delay_ms: store.effective_typing_delay(slot),
        }
    };

    let label = if slot.secret {
        slot.label.clone()
    } else if slot.label.is_empty() {
        auto_label(&slot.body)
    } else {
        slot.label.clone()
    };

    let result = inject(
        &slot.body,
        slot.injection_mode,
        slot.post_paste,
        config,
        slot.secret,
    );

    let toast = if result.is_ok() {
        format!(
            "✓ [{:02} {}] 붙여넣기 완료",
            slot.key,
            if label.is_empty() { "—" } else { &label }
        )
    } else {
        format!("✗ [{:02}] 실패", slot.key)
    };

    osd::emit_pulse(
        app,
        OsdPulsePayload {
            layer: slot.layer,
            key: slot.key,
            label: if slot.secret {
                slot.label.clone()
            } else {
                label
            },
            toast,
        },
    );

    result
}

pub fn on_shortcut_fired(app: &AppHandle, trigger: &str, state: ShortcutState) {
    if state != ShortcutState::Pressed {
        return;
    }
    let app = app.clone();
    let trigger = trigger.to_string();
    std::thread::spawn(move || {
        if let Err(e) = inject_by_trigger(&app, &trigger) {
            eprintln!("[shortcut] inject failed ({trigger}): {e}");
        }
    });
}

pub fn load_persisted(app: &AppHandle, state: &AppState) {
    match persist::load(app) {
        Ok(Some(mut store)) => {
            ensure_default_profile(&mut store);
            sync_active_slots(&mut store);
            if let Ok(mut guard) = state.store.lock() {
                *guard = store;
            }
        }
        Ok(None) => {}
        Err(e) => eprintln!("[persist] load: {e}"),
    }
}

fn persist_store(app: &AppHandle, state: &AppState) -> Result<String, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    ensure_default_profile(&mut store);
    sync_active_slots(&mut store);
    let path = persist::save(app, &store)?;
    Ok(path.display().to_string())
}

fn auto_label(body: &str) -> String {
    let first = body.lines().next().unwrap_or(body);
    first
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(8)
        .collect()
}

