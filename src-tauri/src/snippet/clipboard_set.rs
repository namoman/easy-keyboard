//! Clipboard write helpers. Prefer ExcludeClipboardContentFromMonitorProcessing (pitfall 6).

use arboard::Clipboard;

/// Set UTF-8 text; on Windows also stamp exclude-from-monitor format when possible.
pub fn set_text_excluded(clipboard: &mut Clipboard, text: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        if win::set_text_with_exclude(text).is_ok() {
            return Ok(());
        }
    }
    clipboard
        .set_text(text)
        .map_err(|e| format!("clipboard set: {e}"))
}

#[cfg(windows)]
mod win {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
    };
    use windows_sys::Win32::System::Memory::{
        GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
    };

    const CF_UNICODETEXT: u32 = 13;
    const EXCLUDE_NAME: &str = "ExcludeClipboardContentFromMonitorProcessing";

    pub fn set_text_with_exclude(text: &str) -> Result<(), String> {
        unsafe {
            if OpenClipboard(ptr::null_mut()) == 0 {
                return Err("OpenClipboard failed".into());
            }
            let result = (|| {
                if EmptyClipboard() == 0 {
                    return Err("EmptyClipboard failed".into());
                }
                let exclude = RegisterClipboardFormatW(wide(EXCLUDE_NAME).as_ptr());
                if exclude != 0 {
                    let marker = GlobalAlloc(GMEM_MOVEABLE, 1);
                    if !marker.is_null() {
                        let p = GlobalLock(marker);
                        if !p.is_null() {
                            *(p as *mut u8) = 0;
                            GlobalUnlock(marker);
                            let _ = SetClipboardData(exclude, marker as HANDLE);
                        }
                    }
                }
                let wide_text: Vec<u16> = OsStr::new(text)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                let bytes = wide_text.len() * 2;
                let hmem = GlobalAlloc(GMEM_MOVEABLE, bytes);
                if hmem.is_null() {
                    return Err("GlobalAlloc failed".into());
                }
                let ptr_mem = GlobalLock(hmem);
                if ptr_mem.is_null() {
                    return Err("GlobalLock failed".into());
                }
                std::ptr::copy_nonoverlapping(
                    wide_text.as_ptr() as *const u8,
                    ptr_mem as *mut u8,
                    bytes,
                );
                GlobalUnlock(hmem);
                if SetClipboardData(CF_UNICODETEXT, hmem as HANDLE).is_null() {
                    return Err("SetClipboardData failed".into());
                }
                Ok(())
            })();
            CloseClipboard();
            result
        }
    }

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }
}
