//! Minimal `{{date}}` / `{{time}}` / `{{clipboard}}` substitution (Phase 1).
//! Full tag parser (`{{cursor}}`, `{{tab}}`, …) arrives in Phase 4.

/// Replace Phase-1 template tokens. `clipboard_backup` is the pre-inject clipboard text.
pub fn substitute_vars(template: &str, clipboard_backup: &str) -> String {
    let (date, time) = local_date_time_strings();
    template
        .replace("{{date}}", &date)
        .replace("{{time}}", &time)
        .replace("{{clipboard}}", clipboard_backup)
}

fn local_date_time_strings() -> (String, String) {
    #[cfg(windows)]
    {
        return windows_local_date_time();
    }
    #[cfg(not(windows))]
    {
        fallback_utc_date_time()
    }
}

#[cfg(windows)]
fn windows_local_date_time() -> (String, String) {
    #[repr(C)]
    struct SystemTime {
        year: u16,
        month: u16,
        day_of_week: u16,
        day: u16,
        hour: u16,
        minute: u16,
        second: u16,
        milliseconds: u16,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetLocalTime(lp_system_time: *mut SystemTime);
    }

    let mut st = SystemTime {
        year: 0,
        month: 0,
        day_of_week: 0,
        day: 0,
        hour: 0,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };
    unsafe { GetLocalTime(&mut st) };
    let date = format!("{:04}-{:02}-{:02}", st.year, st.month, st.day);
    let time = format!("{:02}:{:02}:{:02}", st.hour, st.minute, st.second);
    (date, time)
}

#[cfg(not(windows))]
fn fallback_utc_date_time() -> (String, String) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Rough UTC calendar without extra crates (non-Windows CI only).
    let days = secs / 86_400;
    let tod = secs % 86_400;
    let hour = tod / 3600;
    let minute = (tod % 3600) / 60;
    let second = tod % 60;
    let (year, month, day) = civil_from_days(days as i64);
    (
        format!("{year:04}-{month:02}-{day:02}"),
        format!("{hour:02}:{minute:02}:{second:02}"),
    )
}

#[cfg(not(windows))]
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    // Howard Hinnant civil_from_days (UTC).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}
