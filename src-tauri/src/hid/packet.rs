//! 65-byte HID report framing for `514C:8850` (pitfall 4).
//! Clean-room from MINIKeyboard Web/Win protocol notes — not Macrocli source.

pub const VID: u16 = 0x514C;
pub const PID: u16 = 0x8850;
pub const REPORT_ID: u8 = 0x03;
pub const REPORT_LEN: usize = 65;
pub const KIND_KEYS: u8 = 0x01;
pub const MAX_ENTRIES: usize = 18;
pub const RECORDS_PER_LAYER: u8 = 25;
pub const KEYS_PER_LAYER: u8 = 16;
pub const COMMIT_PAUSE_MS: u64 = 200;

/// Report ID + 64-byte zero payload.
pub type Report = [u8; REPORT_LEN];

pub fn empty_report() -> Report {
    let mut report = [0u8; REPORT_LEN];
    report[0] = REPORT_ID;
    report
}

/// Commit / end-program: `03 fd fe ff` + zero pad.
pub fn commit_report() -> Report {
    let mut report = empty_report();
    report[1] = 0xfd;
    report[2] = 0xfe;
    report[3] = 0xff;
    report
}

/// Identify: `03 fb fb fb`.
#[allow(dead_code)]
pub fn identify_report() -> Report {
    let mut report = empty_report();
    report[1] = 0xfb;
    report[2] = 0xfb;
    report[3] = 0xfb;
    report
}

/// Standard-format layer read: `03 fa 19 00 <layer_1based>`.
pub fn read_layer_request(layer_1based: u8) -> Report {
    let mut report = empty_report();
    report[1] = 0xfa;
    report[2] = RECORDS_PER_LAYER;
    report[3] = 0x00;
    report[4] = layer_1based;
    report
}

/// Write one key-macro record. `entries` = `(delay_ms_be, hid_or_modifier_code)`.
pub fn bind_keys_report(
    record_id: u8,
    layer_1based: u8,
    entries: &[(u16, u8)],
) -> Result<Report, String> {
    if !(1..=RECORDS_PER_LAYER).contains(&record_id) {
        return Err(format!("record_id {record_id} out of 1..={RECORDS_PER_LAYER}"));
    }
    if !(1..=3).contains(&layer_1based) {
        return Err(format!("layer {layer_1based} out of 1..=3"));
    }
    if entries.len() > MAX_ENTRIES {
        return Err(format!("too many entries (max {MAX_ENTRIES})"));
    }

    let mut report = empty_report();
    report[1] = 0xfd;
    report[2] = record_id;
    report[3] = layer_1based;
    report[4] = KIND_KEYS;
    report[5] = 0x00;
    report[6] = entries.len() as u8;
    for (index, (delay, code)) in entries.iter().enumerate() {
        let base = 7 + 3 * index;
        report[base] = (delay >> 8) as u8;
        report[base + 1] = (*delay & 0xff) as u8;
        report[base + 2] = *code;
    }
    Ok(report)
}

/// HID usage / protocol modifier codes extracted from a bound report (with Report ID).
pub fn parse_entry_codes(report: &Report) -> Vec<u8> {
    if report[0] != REPORT_ID {
        return Vec::new();
    }
    // Accept write echo (`fd`) or some firmwares' `fa` read framing.
    if report[1] != 0xfd && report[1] != 0xfa {
        return Vec::new();
    }
    let count = report[6].min(MAX_ENTRIES as u8) as usize;
    let mut codes = Vec::with_capacity(count);
    for index in 0..count {
        let base = 7 + 3 * index;
        if base + 2 >= REPORT_LEN {
            break;
        }
        let code = report[base + 2];
        if code != 0 {
            codes.push(code);
        }
    }
    codes
}

pub fn entry_codes_match(expected: &[(u16, u8)], actual: &[u8]) -> bool {
    let want: Vec<u8> = expected.iter().map(|(_, c)| *c).collect();
    want == actual
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_is_65_with_header() {
        let r = commit_report();
        assert_eq!(r.len(), 65);
        assert_eq!(&r[0..4], &[0x03, 0xfd, 0xfe, 0xff]);
        assert!(r[4..].iter().all(|&b| b == 0));
    }

    #[test]
    fn bind_single_f13() {
        let r = bind_keys_report(1, 1, &[(0, 0x68)]).unwrap();
        assert_eq!(r[0], 0x03);
        assert_eq!(r[1], 0xfd);
        assert_eq!(r[2], 1);
        assert_eq!(r[3], 1);
        assert_eq!(r[4], KIND_KEYS);
        assert_eq!(r[6], 1);
        assert_eq!(&r[7..10], &[0x00, 0x00, 0x68]);
        assert_eq!(parse_entry_codes(&r), vec![0x68]);
    }

    #[test]
    fn bind_ctrl_alt_shift_a() {
        let entries = [(0u16, 0xf1), (0, 0xf3), (0, 0xf2), (0, 0x04)];
        let r = bind_keys_report(3, 2, &entries).unwrap();
        assert_eq!(r[6], 4);
        assert_eq!(parse_entry_codes(&r), vec![0xf1, 0xf3, 0xf2, 0x04]);
        assert!(entry_codes_match(&entries, &parse_entry_codes(&r)));
    }
}
