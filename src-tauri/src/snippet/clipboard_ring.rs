//! RAM-only clipboard history (max 20). Honours IS_INJECTING (pitfall 6).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::injector::{inject, is_injecting, InjectConfig};
use super::store::{InjectionMode, PostPasteAction};

pub const RING_MAX: usize = 20;
pub const WHEEL_PREV: &str = "CommandOrControl+Alt+Shift+PageUp";
pub const WHEEL_NEXT: &str = "CommandOrControl+Alt+Shift+PageDown";
pub const WHEEL_PASTE: &str = "CommandOrControl+Alt+Shift+Enter";

static RING_STARTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Default)]
struct RingState {
    items: VecDeque<String>,
    cursor: usize,
    last_seen: String,
}

fn ring() -> &'static Mutex<RingState> {
    static RING: OnceLock<Mutex<RingState>> = OnceLock::new();
    RING.get_or_init(|| Mutex::new(RingState::default()))
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WheelView {
    pub items: Vec<String>,
    pub cursor: usize,
    pub preview: String,
}

pub fn start_monitor(app: &AppHandle) {
    if RING_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(300));
        if is_injecting() {
            continue;
        }
        let text = match read_clipboard() {
            Some(t) if !t.trim().is_empty() => t,
            _ => continue,
        };
        let mut guard = match ring().lock() {
            Ok(g) => g,
            Err(_) => continue,
        };
        if text == guard.last_seen {
            continue;
        }
        guard.last_seen = text.clone();
        if guard.items.front().map(|s| s.as_str()) == Some(text.as_str()) {
            continue;
        }
        guard.items.push_front(text);
        while guard.items.len() > RING_MAX {
            guard.items.pop_back();
        }
        guard.cursor = 0;
        let _ = &app;
    });
}

fn read_clipboard() -> Option<String> {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut c| c.get_text().ok())
}

pub fn snapshot() -> WheelView {
    let guard = ring().lock().unwrap_or_else(|e| e.into_inner());
    let preview = guard
        .items
        .get(guard.cursor)
        .cloned()
        .unwrap_or_default();
    WheelView {
        items: guard.items.iter().cloned().collect(),
        cursor: guard.cursor,
        preview,
    }
}

pub fn rotate_prev(app: &AppHandle) -> WheelView {
    {
        let mut g = ring().lock().unwrap_or_else(|e| e.into_inner());
        if !g.items.is_empty() {
            g.cursor = (g.cursor + 1).min(g.items.len().saturating_sub(1));
        }
    }
    emit_wheel(app)
}

pub fn rotate_next(app: &AppHandle) -> WheelView {
    {
        let mut g = ring().lock().unwrap_or_else(|e| e.into_inner());
        if !g.items.is_empty() && g.cursor > 0 {
            g.cursor -= 1;
        }
    }
    emit_wheel(app)
}

pub fn paste_selected(app: &AppHandle) -> Result<WheelView, String> {
    let text = {
        let g = ring().lock().map_err(|e| e.to_string())?;
        g.items.get(g.cursor).cloned().unwrap_or_default()
    };
    if text.is_empty() {
        return Ok(snapshot());
    }
    inject(
        &text,
        InjectionMode::ClipboardPlain,
        PostPasteAction::None,
        InjectConfig::default(),
        false,
    )?;
    let view = snapshot();
    let _ = app.emit("app://clipboard-wheel", &view);
    Ok(view)
}

fn emit_wheel(app: &AppHandle) -> WheelView {
    let view = snapshot();
    let _ = app.emit("app://clipboard-wheel", &view);
    view
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_caps_at_20() {
        let mut g = ring().lock().unwrap();
        g.items.clear();
        for i in 0..25 {
            g.items.push_front(format!("t{i}"));
            while g.items.len() > RING_MAX {
                g.items.pop_back();
            }
        }
        assert_eq!(g.items.len(), RING_MAX);
    }
}
