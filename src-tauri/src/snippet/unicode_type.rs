//! SimulatedTyping — Hangul-safe UNICODE path (pitfall 5).
//! Uses `enigo::Keyboard::text`, which on Windows emits KEYEVENTF_UNICODE
//! (UTF-16 units), never VK-mapped Hangul.

use std::thread;
use std::time::Duration;

use enigo::{Enigo, Keyboard, Settings};

/// Type text one scalar at a time with configurable delay (5–20ms).
pub fn type_text_unicode(text: &str, delay_ms: u64) -> Result<(), String> {
    let delay = delay_ms.clamp(5, 20);
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("enigo: {e}"))?;
    for ch in text.chars() {
        // Surrogate pairs: enigo encodes the char to UTF-16 UNICODE inputs.
        let s = ch.to_string();
        enigo
            .text(&s)
            .map_err(|e| format!("UNICODE type: {e}"))?;
        thread::sleep(Duration::from_millis(delay));
    }
    Ok(())
}
