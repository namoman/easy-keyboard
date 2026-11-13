//! Second window: OSD overlay 260×310 + Ghost Mode.

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

static OSD_VISIBLE: AtomicBool = AtomicBool::new(false);
static GHOST_MODE: AtomicBool = AtomicBool::new(false);

const OSD_LABEL: &str = "osd";
const WIDTH: f64 = 260.0;
const HEIGHT: f64 = 310.0;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OsdPulsePayload {
    pub layer: u8,
    pub key: u8,
    pub label: String,
    pub toast: String,
}

pub fn ensure_window(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window(OSD_LABEL).is_some() {
        return Ok(());
    }
    WebviewWindowBuilder::new(app, OSD_LABEL, WebviewUrl::App("osd".into()))
        .title("EasyKeyboard OSD")
        .inner_size(WIDTH, HEIGHT)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .focused(false)
        .build()
        .map_err(|e| format!("osd window: {e}"))?;
    Ok(())
}

pub fn is_visible() -> bool {
    OSD_VISIBLE.load(Ordering::SeqCst)
}

pub fn is_ghost() -> bool {
    GHOST_MODE.load(Ordering::SeqCst)
}

pub fn set_visible(app: &AppHandle, visible: bool) -> Result<bool, String> {
    ensure_window(app)?;
    let win = app
        .get_webview_window(OSD_LABEL)
        .ok_or_else(|| "osd window missing".to_string())?;
    if visible {
        win.show().map_err(|e| e.to_string())?;
        let _ = win.set_always_on_top(true);
    } else {
        win.hide().map_err(|e| e.to_string())?;
    }
    OSD_VISIBLE.store(visible, Ordering::SeqCst);
    let _ = app.emit("osd-visibility", visible);
    Ok(visible)
}

pub fn toggle_visible(app: &AppHandle) -> Result<bool, String> {
    let next = !is_visible();
    set_visible(app, next)
}

pub fn set_ghost(app: &AppHandle, ghost: bool) -> Result<bool, String> {
    ensure_window(app)?;
    let win = app
        .get_webview_window(OSD_LABEL)
        .ok_or_else(|| "osd window missing".to_string())?;
    win.set_ignore_cursor_events(ghost)
        .map_err(|e| format!("ignore cursor: {e}"))?;
    // Opacity is applied in the OSD webview (15%) via `osd-ghost` event — portable.
    GHOST_MODE.store(ghost, Ordering::SeqCst);
    let _ = app.emit("osd-ghost", ghost);
    Ok(ghost)
}

pub fn toggle_ghost(app: &AppHandle) -> Result<bool, String> {
    set_ghost(app, !is_ghost())
}

pub fn emit_pulse(app: &AppHandle, payload: OsdPulsePayload) {
    let _ = app.emit("osd-pulse", &payload);
}

/// Register Ctrl+Shift+O / Ctrl+Shift+G (global).
pub fn register_hotkeys(app: &AppHandle) -> Result<(), String> {
    let gs = app.global_shortcut();

    let app_o = app.clone();
    gs.on_shortcut("Ctrl+Shift+O", move |app, _s, event| {
        if event.state != ShortcutState::Pressed {
            return;
        }
        if let Err(e) = toggle_visible(app) {
            eprintln!("[osd] toggle: {e}");
        }
        let _ = &app_o;
    })
    .map_err(|e| format!("register Ctrl+Shift+O: {e}"))?;

    gs.on_shortcut("Ctrl+Shift+G", move |app, _s, event| {
        if event.state != ShortcutState::Pressed {
            return;
        }
        if let Err(e) = toggle_ghost(app) {
            eprintln!("[osd] ghost: {e}");
        }
    })
    .map_err(|e| format!("register Ctrl+Shift+G: {e}"))?;

    Ok(())
}
