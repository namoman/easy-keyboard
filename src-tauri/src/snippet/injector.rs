//! Injection routing: ClipboardPlain / ClipboardTerminal / SimulatedTyping.
//! Pitfalls 1 (modifier settle), 2 (restore delay), 5 (UNICODE), 6 (IS_INJECTING).

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use arboard::Clipboard;
use enigo::Key;
use zeroize::Zeroize;

use super::clipboard_set;
use super::keys::{
    clear_modifiers, new_enigo, send_ctrl_v, send_key, send_left_n, send_shift_insert,
};
use super::parser::{self, Action, Plan};
use super::store::{InjectionMode, PostPasteAction};
use super::unicode_type::type_text_unicode;

/// Guard for clipboard watchers / ring (pitfall 6). SeqCst required.
pub static IS_INJECTING: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)] // Phase 5 clipboard_ring polls this.
pub fn is_injecting() -> bool {
    IS_INJECTING.load(Ordering::SeqCst)
}

#[derive(Debug, Clone, Copy)]
pub struct InjectConfig {
    pub modifier_settle_ms: u64,
    pub restore_delay_ms: u64,
    pub typing_delay_ms: u64,
}

impl Default for InjectConfig {
    fn default() -> Self {
        Self {
            modifier_settle_ms: 40,
            restore_delay_ms: 100,
            typing_delay_ms: 10,
        }
    }
}

/// Route by mode, run parser plan, optional secret zeroize of owned body.
pub fn inject(
    body: &str,
    mode: InjectionMode,
    post_paste: PostPasteAction,
    config: InjectConfig,
    secret: bool,
) -> Result<(), String> {
    if body.is_empty() {
        return Ok(());
    }

    IS_INJECTING.store(true, Ordering::SeqCst);
    let mut owned = body.to_string();
    let result = (|| {
        let mut clipboard = Clipboard::new().map_err(|e| format!("clipboard open: {e}"))?;
        let backup = clipboard.get_text().unwrap_or_default();
        let plan = parser::parse(&owned, &backup);

        thread::sleep(Duration::from_millis(config.modifier_settle_ms.clamp(30, 50)));

        match mode {
            InjectionMode::ClipboardPlain => {
                run_clipboard_plan(&plan, &mut clipboard, &backup, config, false)?
            }
            InjectionMode::ClipboardTerminal => {
                run_clipboard_plan(&plan, &mut clipboard, &backup, config, true)?
            }
            InjectionMode::SimulatedTyping => run_typing_plan(&plan, config)?,
        }

        apply_post_paste(post_paste)?;
        Ok(())
    })();

    if secret {
        owned.zeroize();
    }

    thread::sleep(Duration::from_millis(50));
    IS_INJECTING.store(false, Ordering::SeqCst);
    result
}

fn run_clipboard_plan(
    plan: &Plan,
    clipboard: &mut Clipboard,
    backup: &str,
    config: InjectConfig,
    terminal: bool,
) -> Result<(), String> {
    let mut enigo = new_enigo()?;
    for (idx, action) in plan.actions.iter().enumerate() {
        match action {
            Action::Text(text) => {
                if text.is_empty() {
                    continue;
                }
                clipboard_set::set_text_excluded(clipboard, text)?;
                thread::sleep(Duration::from_millis(config.modifier_settle_ms.clamp(30, 50)));
                if terminal {
                    send_shift_insert(&mut enigo)?;
                } else {
                    send_ctrl_v(&mut enigo)?;
                }
                thread::sleep(Duration::from_millis(config.restore_delay_ms.clamp(80, 120)));
                if let Err(e) = clipboard_set::set_text_excluded(clipboard, backup) {
                    eprintln!("[injector] clipboard restore failed: {e}");
                }
                maybe_cursor(&mut enigo, plan, idx)?;
            }
            Action::Tab => {
                thread::sleep(Duration::from_millis(50));
                send_key(&mut enigo, Key::Tab)?;
            }
            Action::Enter => send_key(&mut enigo, Key::Return)?,
            Action::Wait(ms) => thread::sleep(Duration::from_millis(*ms)),
        }
    }
    Ok(())
}

fn run_typing_plan(plan: &Plan, config: InjectConfig) -> Result<(), String> {
    let mut enigo = new_enigo()?;
    clear_modifiers(&mut enigo)?;
    for (idx, action) in plan.actions.iter().enumerate() {
        match action {
            Action::Text(text) => {
                type_text_unicode(text, config.typing_delay_ms)?;
                maybe_cursor(&mut enigo, plan, idx)?;
            }
            Action::Tab => {
                thread::sleep(Duration::from_millis(50));
                send_key(&mut enigo, Key::Tab)?;
            }
            Action::Enter => send_key(&mut enigo, Key::Return)?,
            Action::Wait(ms) => thread::sleep(Duration::from_millis(*ms)),
        }
    }
    Ok(())
}

fn maybe_cursor(
    enigo: &mut enigo::Enigo,
    plan: &Plan,
    action_idx: usize,
) -> Result<(), String> {
    if plan.cursor_after_action == Some(action_idx) {
        if let Some(n) = plan.cursor_left {
            send_left_n(enigo, n)?;
        }
    }
    Ok(())
}

fn apply_post_paste(post_paste: PostPasteAction) -> Result<(), String> {
    let mut enigo = new_enigo()?;
    match post_paste {
        PostPasteAction::None => Ok(()),
        PostPasteAction::Enter => send_key(&mut enigo, Key::Return),
        PostPasteAction::Tab => send_key(&mut enigo, Key::Tab),
    }
}
