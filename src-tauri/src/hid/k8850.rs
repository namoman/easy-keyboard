//! Open `514C:8850` config collection (vendor usage page), bind / read / verify.

use std::thread;
use std::time::Duration;

use hidapi::{DeviceInfo, HidApi, HidDevice};
use serde::Serialize;

use super::packet::{
    bind_keys_report, commit_report, empty_report, entry_codes_match, identify_report,
    parse_entry_codes, read_layer_request, Report, COMMIT_PAUSE_MS, KEYS_PER_LAYER, PID,
    RECORDS_PER_LAYER, REPORT_ID, REPORT_LEN, VID,
};
use super::triggers::{trigger_accel, trigger_entries, TriggerProfile};

const READ_TIMEOUT_MS: i32 = 150;
const KEYBOARD_USAGE_PAGE: u16 = 0x01;
const KEYBOARD_USAGE: u16 = 0x06;
const VENDOR_USAGE_PAGE_MIN: u16 = 0xFF00;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    pub connected: bool,
    pub path: Option<String>,
    pub interface: Option<i32>,
    pub usage_page: Option<u16>,
    pub usage: Option<u16>,
    pub product: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerRecord {
    pub record_id: u8,
    pub codes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteVerifyResult {
    pub ok: bool,
    pub accel: String,
    pub written_codes: Vec<u8>,
    pub readback_codes: Vec<u8>,
}

pub struct K8850Device {
    _api: HidApi,
    device: HidDevice,
    path: String,
    interface: i32,
    usage_page: u16,
    usage: u16,
    product: Option<String>,
}

impl K8850Device {
    pub fn open() -> Result<Self, String> {
        let api = HidApi::new().map_err(|e| format!("hidapi init: {e}"))?;
        let info = select_config_interface(&api)?;
        let path = info.path().to_string_lossy().into_owned();
        let interface = info.interface_number();
        let usage_page = info.usage_page();
        let usage = info.usage();
        let product = info.product_string().map(str::to_owned);
        let device = api
            .open_path(info.path())
            .map_err(|e| format!("open config interface failed: {e}"))?;
        let mut opened = Self {
            _api: api,
            device,
            path,
            interface,
            usage_page,
            usage,
            product,
        };
        // Best-effort identity probe; ignore if the pad is busy / silent.
        let _ = opened.identify();
        Ok(opened)
    }

    pub fn status(&self) -> DeviceStatus {
        DeviceStatus {
            connected: true,
            path: Some(self.path.clone()),
            interface: Some(self.interface),
            usage_page: Some(self.usage_page),
            usage: Some(self.usage),
            product: self.product.clone(),
        }
    }

    pub fn identify(&mut self) -> Result<[u8; 5], String> {
        self.write_report(&identify_report())?;
        let reply = self.read_report()?;
        Ok([reply[0], reply[1], reply[2], reply[3], reply[4]])
    }

    pub fn bind_key(
        &mut self,
        layer_1based: u8,
        key_1based: u8,
        profile: TriggerProfile,
    ) -> Result<WriteVerifyResult, String> {
        let entries = trigger_entries(profile, key_1based)?;
        let accel = trigger_accel(profile, key_1based)?;
        let report = bind_keys_report(key_1based, layer_1based, &entries)?;
        self.write_report(&report)?;
        self.write_report(&commit_report())?;
        thread::sleep(Duration::from_millis(COMMIT_PAUSE_MS));

        let readback = self.read_single_record(layer_1based, key_1based)?;
        let written_codes: Vec<u8> = entries.iter().map(|(_, c)| *c).collect();
        let ok = entry_codes_match(&entries, &readback);
        Ok(WriteVerifyResult {
            ok,
            accel,
            written_codes,
            readback_codes: readback,
        })
    }

    pub fn read_layer(&mut self, layer_1based: u8) -> Result<Vec<LayerRecord>, String> {
        if !(1..=3).contains(&layer_1based) {
            return Err(format!("layer {layer_1based} out of 1..=3"));
        }
        self.write_report(&read_layer_request(layer_1based))?;
        let mut out = Vec::new();
        for record_id in 1..=RECORDS_PER_LAYER {
            match self.read_report() {
                Ok(report) => {
                    let codes = parse_entry_codes(&report);
                    // Prefer device-reported id when present.
                    let id = if report[1] == 0xfd || report[1] == 0xfa {
                        if report[2] != 0 {
                            report[2]
                        } else {
                            record_id
                        }
                    } else {
                        record_id
                    };
                    out.push(LayerRecord {
                        record_id: id,
                        codes,
                    });
                }
                Err(_) if record_id == 1 => {
                    return Err("no reply for layer read".into());
                }
                Err(_) => break,
            }
        }
        Ok(out)
    }

    pub fn verify_write(
        &mut self,
        layer_1based: u8,
        key_1based: u8,
        profile: TriggerProfile,
    ) -> Result<bool, String> {
        let entries = trigger_entries(profile, key_1based)?;
        let actual = self.read_single_record(layer_1based, key_1based)?;
        Ok(entry_codes_match(&entries, &actual))
    }

    fn read_single_record(&mut self, layer_1based: u8, key_1based: u8) -> Result<Vec<u8>, String> {
        let layer = self.read_layer(layer_1based)?;
        layer
            .into_iter()
            .find(|r| r.record_id == key_1based)
            .map(|r| r.codes)
            .ok_or_else(|| format!("record {key_1based} missing in layer read-back"))
    }

    fn write_report(&mut self, report: &Report) -> Result<(), String> {
        debug_assert_eq!(report.len(), REPORT_LEN);
        debug_assert_eq!(report[0], REPORT_ID);
        self.device
            .write(report)
            .map_err(|e| format!("hid write: {e}"))?;
        Ok(())
    }

    fn read_report(&mut self) -> Result<Report, String> {
        let mut buf = empty_report();
        let n = self
            .device
            .read_timeout(&mut buf, READ_TIMEOUT_MS)
            .map_err(|e| format!("hid read: {e}"))?;
        if n == 0 {
            return Err("hid read timeout".into());
        }
        Ok(buf)
    }
}

/// Prefer vendor-defined usage page; never open the plain keyboard collection blindly.
pub fn select_config_interface(api: &HidApi) -> Result<DeviceInfo, String> {
    let candidates: Vec<&DeviceInfo> = api
        .device_list()
        .filter(|d| d.vendor_id() == VID && d.product_id() == PID)
        .filter(|d| !is_plain_keyboard(d))
        .collect();

    if candidates.is_empty() {
        return Err(format!(
            "device {VID:04X}:{PID:04X} not found (or only keyboard MI present)"
        ));
    }

    let ranked = |d: &&DeviceInfo| -> (u8, i32, i32) {
        let vendor = if d.usage_page() >= VENDOR_USAGE_PAGE_MIN {
            0
        } else {
            1
        };
        // Pitfall 4: config path is typically MI_01 on this pad; MI_00 is often the keyboard.
        let iface_pref = match d.interface_number() {
            1 => 0,
            0 => 1,
            other => 2 + other.unsigned_abs() as i32,
        };
        (vendor, iface_pref, d.interface_number())
    };

    candidates
        .into_iter()
        .min_by_key(ranked)
        .cloned()
        .ok_or_else(|| "no config interface".into())
}

fn is_plain_keyboard(info: &DeviceInfo) -> bool {
    info.usage_page() == KEYBOARD_USAGE_PAGE && info.usage() == KEYBOARD_USAGE
}

pub fn disconnected_status() -> DeviceStatus {
    DeviceStatus {
        connected: false,
        path: None,
        interface: None,
        usage_page: None,
        usage: None,
        product: None,
    }
}

pub fn validate_key(key_1based: u8) -> Result<(), String> {
    if (1..=KEYS_PER_LAYER).contains(&key_1based) {
        Ok(())
    } else {
        Err(format!("key {key_1based} out of 1..={KEYS_PER_LAYER}"))
    }
}
