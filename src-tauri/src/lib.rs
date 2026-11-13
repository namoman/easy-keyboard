//! EasyKeyboard host entry. Phase 5: profiles + clipboard ring + .ekb.

mod commands;
mod commands_ext;
mod context;
mod hid;
mod osd;
mod shortcuts;
mod snippet;
mod state;
mod tray;

use tauri::{Manager, WindowEvent};

use hid::{
    hid_connect, hid_disconnect, hid_read_layer, hid_status, hid_trigger_accel, hid_verify_trigger,
    hid_write_trigger, HidState,
};
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState::default())
        .manage(HidState::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_slots,
            commands::save_snippet,
            commands::register_trigger,
            commands::paste_test,
            commands::get_clipboard_text,
            commands::set_timing,
            commands::persist_snippets,
            commands::osd_toggle,
            commands::osd_set_visible,
            commands::osd_toggle_ghost,
            commands::osd_status,
            commands::osd_save_slot,
            commands_ext::list_profiles,
            commands_ext::set_profile_pin,
            commands_ext::switch_app_profile,
            commands_ext::upsert_profile_rule,
            commands_ext::delete_profile,
            commands_ext::clipboard_wheel_snapshot,
            commands_ext::clipboard_wheel_prev,
            commands_ext::clipboard_wheel_next,
            commands_ext::clipboard_wheel_paste,
            commands_ext::export_ekb,
            commands_ext::import_ekb,
            commands_ext::export_clean_json,
            commands_ext::import_clean_json,
            hid_connect,
            hid_disconnect,
            hid_status,
            hid_write_trigger,
            hid_verify_trigger,
            hid_read_layer,
            hid_trigger_accel,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let state = handle.state::<AppState>();
            commands::load_persisted(&handle, &*state);
            tray::setup(app.handle())?;
            if let Err(e) = osd::ensure_window(app.handle()) {
                eprintln!("[osd] ensure: {e}");
            }
            if let Err(e) = shortcuts::refresh_all(app.handle()) {
                eprintln!("[shortcuts] initial register: {e}");
            }
            context::start_watcher(app.handle());
            snippet::clipboard_ring::start_monitor(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running EasyKeyboard");
}
