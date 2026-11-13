//! 48 slots (layer 1–3 × 16 keys). Persistence via `crypto` / `persist`.

use serde::{Deserialize, Serialize};

use super::profiles::{AppProfile, DEFAULT_PROFILE_ID};

pub const SLOT_COUNT: usize = 48;
pub const KEYS_PER_LAYER: usize = 16;
pub const LAYER_COUNT: usize = 3;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum PostPasteAction {
    #[default]
    None,
    Enter,
    Tab,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum InjectionMode {
    #[default]
    ClipboardPlain,
    ClipboardTerminal,
    SimulatedTyping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetSlot {
    pub layer: u8,
    pub key: u8,
    pub label: String,
    pub body: String,
    pub secret: bool,
    pub post_paste: PostPasteAction,
    /// Global-shortcut accelerator, e.g. `"F13"`. Empty = unregistered.
    pub trigger: String,
    #[serde(default)]
    pub injection_mode: InjectionMode,
    /// Per-slot SimulatedTyping delay (5–20ms). 0 = use store default.
    #[serde(default)]
    pub typing_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetStore {
    pub slots: Vec<SnippetSlot>,
    /// Clipboard restore delay after paste (pitfall 2). Clamped 80–120.
    pub restore_delay_ms: u64,
    /// Modifier settle before paste (pitfall 1). Clamped 30–50.
    pub modifier_settle_ms: u64,
    /// Default SimulatedTyping delay. Clamped 5–20.
    #[serde(default = "default_typing_delay")]
    pub typing_delay_ms: u64,
    /// App-aware profile banks (Phase 5). Empty → migrated to Default on load.
    #[serde(default)]
    pub profiles: Vec<AppProfile>,
    #[serde(default = "default_active_profile")]
    pub active_profile_id: String,
    /// Pin lock: stop automatic profile switching.
    #[serde(default)]
    pub pin_locked: bool,
}

fn default_typing_delay() -> u64 {
    10
}

fn default_active_profile() -> String {
    DEFAULT_PROFILE_ID.into()
}

impl Default for SnippetStore {
    fn default() -> Self {
        let mut slots = Vec::with_capacity(SLOT_COUNT);
        for layer in 1..=LAYER_COUNT as u8 {
            for key in 1..=KEYS_PER_LAYER as u8 {
                slots.push(SnippetSlot {
                    layer,
                    key,
                    label: String::new(),
                    body: String::new(),
                    secret: false,
                    post_paste: PostPasteAction::None,
                    trigger: default_trigger_for(layer, key),
                    injection_mode: InjectionMode::ClipboardPlain,
                    typing_delay_ms: 0,
                });
            }
        }
        let profiles = vec![AppProfile::default_from_slots(slots.clone())];
        Self {
            slots,
            restore_delay_ms: 100,
            modifier_settle_ms: 40,
            typing_delay_ms: default_typing_delay(),
            profiles,
            active_profile_id: default_active_profile(),
            pin_locked: false,
        }
    }
}

impl SnippetStore {
    pub fn find_by_trigger(&self, trigger: &str) -> Option<&SnippetSlot> {
        let norm = trigger.trim();
        if norm.is_empty() {
            return None;
        }
        self.slots
            .iter()
            .find(|s| s.trigger.eq_ignore_ascii_case(norm))
    }

    pub fn set_restore_delay_ms(&mut self, ms: u64) {
        self.restore_delay_ms = ms.clamp(80, 120);
    }

    pub fn set_modifier_settle_ms(&mut self, ms: u64) {
        self.modifier_settle_ms = ms.clamp(30, 50);
    }

    pub fn set_typing_delay_ms(&mut self, ms: u64) {
        self.typing_delay_ms = ms.clamp(5, 20);
    }

    pub fn effective_typing_delay(&self, slot: &SnippetSlot) -> u64 {
        if slot.typing_delay_ms == 0 {
            self.typing_delay_ms.clamp(5, 20)
        } else {
            slot.typing_delay_ms.clamp(5, 20)
        }
    }
}

/// Layer 1 keys 01–12 → F13–F24. Other slots start unregistered.
pub fn default_trigger_for(layer: u8, key: u8) -> String {
    if layer == 1 && (1..=12).contains(&key) {
        format!("F{}", 12 + key)
    } else {
        String::new()
    }
}

pub fn slot_index(layer: u8, key: u8) -> Option<usize> {
    if !(1..=LAYER_COUNT as u8).contains(&layer) || !(1..=KEYS_PER_LAYER as u8).contains(&key) {
        return None;
    }
    Some(((layer as usize) - 1) * KEYS_PER_LAYER + ((key as usize) - 1))
}
