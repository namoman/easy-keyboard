//! Poll foreground exe/title (150ms). Emit profile switch + elevated badge.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::snippet::profiles::{
    ensure_default_profile, match_profile, switch_profile, DEFAULT_PROFILE_ID,
};
use crate::state::AppState;

static WATCHER_STARTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForegroundInfo {
    pub exe: String,
    pub title: String,
    pub elevated: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSwitchedPayload {
    pub profile_id: String,
    pub name: String,
    pub exe: String,
    pub title: String,
}

pub fn start_watcher(app: &AppHandle) {
    if WATCHER_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    thread::spawn(move || {
        let mut last_key = String::new();
        let mut last_elevated = false;
        loop {
            thread::sleep(Duration::from_millis(150));
            let info = read_foreground();
            if info.elevated != last_elevated {
                last_elevated = info.elevated;
                let _ = app.emit("app://foreground-elevated", info.elevated);
            }
            let key = format!("{}|{}", info.exe.to_ascii_lowercase(), info.title);
            if key == last_key {
                continue;
            }
            last_key = key;
            let _ = app.emit("app://foreground-changed", &info);
            apply_auto_switch(&app, &info);
        }
    });
}

fn apply_auto_switch(app: &AppHandle, info: &ForegroundInfo) {
    let state = app.state::<AppState>();
    let switched = {
        let mut store = match state.store.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if store.pin_locked {
            return;
        }
        ensure_default_profile(&mut store);
        let target_id = match_profile(&store.profiles, &info.exe, &info.title)
            .map(|p| p.rule.profile_id.clone())
            .unwrap_or_else(|| DEFAULT_PROFILE_ID.into());
        if target_id == store.active_profile_id {
            return;
        }
        match switch_profile(&mut store, &target_id) {
            Ok(p) => Some(ProfileSwitchedPayload {
                profile_id: p.rule.profile_id.clone(),
                name: p.rule.name.clone(),
                exe: info.exe.clone(),
                title: info.title.clone(),
            }),
            Err(e) => {
                eprintln!("[watcher] switch: {e}");
                None
            }
        }
    };
    if let Some(payload) = switched {
        let _ = crate::shortcuts::refresh_all(app);
        let _ = app.emit("app://profile-switched", &payload);
    }
}

pub fn read_foreground() -> ForegroundInfo {
    #[cfg(windows)]
    {
        return win::read();
    }
    #[cfg(not(windows))]
    {
        ForegroundInfo {
            exe: String::new(),
            title: String::new(),
            elevated: false,
        }
    }
}

#[cfg(windows)]
mod win {
    use super::ForegroundInfo;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::ptr;

    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, HWND, MAX_PATH};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, OpenProcessToken, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    pub fn read() -> ForegroundInfo {
        unsafe {
            let hwnd: HWND = GetForegroundWindow();
            if hwnd.is_null() {
                return empty();
            }
            let title = window_title(hwnd);
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == 0 {
                return ForegroundInfo {
                    exe: String::new(),
                    title,
                    elevated: false,
                };
            }
            let (exe, elevated) = process_info(pid);
            ForegroundInfo {
                exe,
                title,
                elevated,
            }
        }
    }

    fn empty() -> ForegroundInfo {
        ForegroundInfo {
            exe: String::new(),
            title: String::new(),
            elevated: false,
        }
    }

    unsafe fn window_title(hwnd: HWND) -> String {
        let mut buf = [0u16; 512];
        let n = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        if n <= 0 {
            return String::new();
        }
        OsString::from_wide(&buf[..n as usize])
            .to_string_lossy()
            .into_owned()
    }

    unsafe fn process_info(pid: u32) -> (String, bool) {
        let proc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if proc.is_null() {
            return (String::new(), false);
        }
        let mut path_buf = [0u16; MAX_PATH as usize];
        let mut size = path_buf.len() as u32;
        let ok = QueryFullProcessImageNameW(proc, 0, path_buf.as_mut_ptr(), &mut size);
        let path = if ok != 0 && size > 0 {
            OsString::from_wide(&path_buf[..size as usize])
                .to_string_lossy()
                .into_owned()
        } else {
            String::new()
        };
        let exe = std::path::Path::new(&path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or(path);
        let elevated = is_elevated(proc);
        CloseHandle(proc);
        (exe, elevated)
    }

    unsafe fn is_elevated(proc: HANDLE) -> bool {
        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(proc, TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elev = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut ret = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elev as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret,
        );
        CloseHandle(token);
        ok != 0 && elev.TokenIsElevated != 0
    }
}
