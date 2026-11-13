//! Dual trigger profiles: Wired/2.4G F13–F24 vs BT Ctrl+Alt+Shift combos.

use serde::{Deserialize, Serialize};

use super::packet::KEYS_PER_LAYER;

/// Protocol modifier hold-codes (F1–F8 style), lowest bit first.
pub const MOD_LCTRL: u8 = 0xf1;
pub const MOD_LSHIFT: u8 = 0xf2;
pub const MOD_LALT: u8 = 0xf3;

/// HID Keyboard usages F13–F24.
pub const HID_F13: u8 = 0x68;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TriggerProfile {
    /// USB-C / 2.4 GHz dongle — F13–F24 (keys 1–12), Ctrl+F13–F16 (13–16).
    WiredDongle,
    /// Bluetooth — Ctrl+Alt+Shift + letter `a`…`p` (always transmitted).
    Bluetooth,
}

impl Default for TriggerProfile {
    fn default() -> Self {
        Self::WiredDongle
    }
}

/// `(delay_ms, code)` sequence for a physical key (1..=16) under `profile`.
pub fn trigger_entries(profile: TriggerProfile, key_1based: u8) -> Result<Vec<(u16, u8)>, String> {
    if !(1..=KEYS_PER_LAYER).contains(&key_1based) {
        return Err(format!("key {key_1based} out of 1..={KEYS_PER_LAYER}"));
    }
    Ok(match profile {
        TriggerProfile::WiredDongle => wired_entries(key_1based),
        TriggerProfile::Bluetooth => bluetooth_entries(key_1based),
    })
}

/// Accelerator string for `global-shortcut` / Phase 1 slot.trigger.
pub fn trigger_accel(profile: TriggerProfile, key_1based: u8) -> Result<String, String> {
    if !(1..=KEYS_PER_LAYER).contains(&key_1based) {
        return Err(format!("key {key_1based} out of 1..={KEYS_PER_LAYER}"));
    }
    Ok(match profile {
        TriggerProfile::WiredDongle => {
            if key_1based <= 12 {
                format!("F{}", 12 + key_1based)
            } else {
                format!("Control+F{}", 12 + (key_1based - 12))
            }
        }
        TriggerProfile::Bluetooth => {
            let letter = bluetooth_letter(key_1based);
            format!("Control+Alt+Shift+{}", letter.to_ascii_uppercase())
        }
    })
}

fn wired_entries(key_1based: u8) -> Vec<(u16, u8)> {
    if key_1based <= 12 {
        vec![(0, HID_F13 + (key_1based - 1))]
    } else {
        let f = HID_F13 + (key_1based - 13);
        vec![(0, MOD_LCTRL), (0, f)]
    }
}

fn bluetooth_entries(key_1based: u8) -> Vec<(u16, u8)> {
    let usage = 0x04 + (key_1based - 1); // a=0x04 … p=0x13
    vec![
        (0, MOD_LCTRL),
        (0, MOD_LALT),
        (0, MOD_LSHIFT),
        (0, usage),
    ]
}

fn bluetooth_letter(key_1based: u8) -> char {
    char::from(b'a' + (key_1based - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wired_f13_and_ctrl_f13() {
        assert_eq!(trigger_entries(TriggerProfile::WiredDongle, 1).unwrap(), vec![(0, 0x68)]);
        assert_eq!(trigger_accel(TriggerProfile::WiredDongle, 1).unwrap(), "F13");
        assert_eq!(
            trigger_entries(TriggerProfile::WiredDongle, 13).unwrap(),
            vec![(0, MOD_LCTRL), (0, 0x68)]
        );
        assert_eq!(
            trigger_accel(TriggerProfile::WiredDongle, 13).unwrap(),
            "Control+F13"
        );
    }

    #[test]
    fn bluetooth_slot_a() {
        let e = trigger_entries(TriggerProfile::Bluetooth, 1).unwrap();
        assert_eq!(e, vec![(0, 0xf1), (0, 0xf3), (0, 0xf2), (0, 0x04)]);
        assert_eq!(
            trigger_accel(TriggerProfile::Bluetooth, 1).unwrap(),
            "Control+Alt+Shift+A"
        );
    }
}
