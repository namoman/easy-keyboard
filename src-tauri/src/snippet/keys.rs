//! Shared enigo key helpers (modifier clear, paste chords, Left N).

use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};

pub fn new_enigo() -> Result<Enigo, String> {
    Enigo::new(&Settings::default()).map_err(|e| format!("enigo: {e}"))
}

pub fn clear_modifiers(enigo: &mut Enigo) -> Result<(), String> {
    enigo
        .key(Key::Alt, Release)
        .map_err(|e| format!("alt up: {e}"))?;
    enigo
        .key(Key::Shift, Release)
        .map_err(|e| format!("shift up: {e}"))?;
    Ok(())
}

pub fn send_ctrl_v(enigo: &mut Enigo) -> Result<(), String> {
    clear_modifiers(enigo)?;
    enigo
        .key(Key::Control, Press)
        .map_err(|e| format!("ctrl down: {e}"))?;
    enigo
        .key(Key::Unicode('v'), Click)
        .map_err(|e| format!("v click: {e}"))?;
    enigo
        .key(Key::Control, Release)
        .map_err(|e| format!("ctrl up: {e}"))?;
    Ok(())
}

/// Terminal paste: Shift+Insert (widely accepted). Fallback path Ctrl+Shift+V unused here.
pub fn send_shift_insert(enigo: &mut Enigo) -> Result<(), String> {
    clear_modifiers(enigo)?;
    enigo
        .key(Key::Shift, Press)
        .map_err(|e| format!("shift down: {e}"))?;
    enigo
        .key(Key::Insert, Click)
        .map_err(|e| format!("insert: {e}"))?;
    enigo
        .key(Key::Shift, Release)
        .map_err(|e| format!("shift up: {e}"))?;
    Ok(())
}

pub fn send_key(enigo: &mut Enigo, key: Key) -> Result<(), String> {
    enigo
        .key(key, Click)
        .map_err(|e| format!("key click: {e}"))
}

pub fn send_left_n(enigo: &mut Enigo, n: usize) -> Result<(), String> {
    for _ in 0..n {
        send_key(enigo, Key::LeftArrow)?;
    }
    Ok(())
}
