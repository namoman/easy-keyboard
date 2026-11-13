//! Global-shortcut registration for snippet triggers + OSD + clipboard wheel.

use std::collections::HashSet;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::commands::on_shortcut_fired;
use crate::osd;
use crate::snippet::clipboard_ring::{self, WHEEL_NEXT, WHEEL_PASTE, WHEEL_PREV};
use crate::state::AppState;

/// Unregister all, then bind every non-empty slot trigger + OSD + wheel chords.
pub fn refresh_all(app: &AppHandle) -> Result<(), String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();

    let triggers: Vec<String> = {
        let state = app.state::<AppState>();
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let mut seen = HashSet::new();
        store
            .slots
            .iter()
            .filter_map(|s| {
                let t = s.trigger.trim();
                if t.is_empty() || !seen.insert(t.to_ascii_uppercase()) {
                    None
                } else {
                    Some(t.to_string())
                }
            })
            .collect()
    };

    for trigger in triggers {
        let label = trigger.clone();
        gs.on_shortcut(trigger.as_str(), move |app, _shortcut, event| {
            on_shortcut_fired(app, &label, event.state);
        })
        .map_err(|e| format!("register {trigger}: {e}"))?;
    }

    osd::register_hotkeys(app)?;
    register_wheel(app)?;
    Ok(())
}

fn register_wheel(app: &AppHandle) -> Result<(), String> {
    let gs = app.global_shortcut();
    gs.on_shortcut(WHEEL_PREV, move |app, _s, event| {
        if event.state == ShortcutState::Pressed {
            let _ = clipboard_ring::rotate_prev(app);
        }
    })
    .map_err(|e| format!("wheel prev: {e}"))?;
    gs.on_shortcut(WHEEL_NEXT, move |app, _s, event| {
        if event.state == ShortcutState::Pressed {
            let _ = clipboard_ring::rotate_next(app);
        }
    })
    .map_err(|e| format!("wheel next: {e}"))?;
    gs.on_shortcut(WHEEL_PASTE, move |app, _s, event| {
        if event.state == ShortcutState::Pressed {
            if let Err(e) = clipboard_ring::paste_selected(app) {
                eprintln!("[wheel] paste: {e}");
            }
        }
    })
    .map_err(|e| format!("wheel paste: {e}"))?;
    Ok(())
}
