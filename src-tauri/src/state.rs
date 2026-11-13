//! Shared app state. HID open handle lives in `hid::HidState` (Phase 2).

use std::sync::Mutex;

use crate::snippet::SnippetStore;

pub struct AppState {
    pub store: Mutex<SnippetStore>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            store: Mutex::new(SnippetStore::default()),
        }
    }
}
