//! Template parser: `{{cursor}}` / `{{tab}}` / `{{enter}}` / `{{wait:ms}}`.

use unicode_segmentation::UnicodeSegmentation;

use super::substitute::substitute_vars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Text(String),
    Tab,
    Enter,
    Wait(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub actions: Vec<Action>,
    /// After injecting the Text at `cursor_after_action`, send this many Left keys.
    pub cursor_left: Option<usize>,
    pub cursor_after_action: Option<usize>,
}

/// Expand vars then tokenize control tags into an inject plan.
pub fn parse(template: &str, clipboard_backup: &str) -> Plan {
    plan_from_expanded(&substitute_vars(template, clipboard_backup))
}

fn plan_from_expanded(s: &str) -> Plan {
    let mut actions: Vec<Action> = Vec::new();
    let mut text_buf = String::new();
    let mut cursor_left: Option<usize> = None;
    let mut cursor_after_action: Option<usize> = None;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0usize;

    while i < chars.len() {
        if let Some((tag, consumed)) = try_tag(&chars[i..]) {
            match tag.as_str() {
                "cursor" => {
                    let after = text_until_next_control(&chars[i + consumed..]);
                    cursor_left = Some(grapheme_units(&after));
                    // Marker removed; continue so post-cursor text joins this buffer.
                    i += consumed;
                    continue;
                }
                "tab" | "enter" => {
                    flush_text(&mut text_buf, &mut actions);
                    if tag == "tab" {
                        actions.push(Action::Tab);
                    } else {
                        actions.push(Action::Enter);
                    }
                    if cursor_left.is_some() && cursor_after_action.is_none() {
                        cursor_after_action = last_text_index(&actions);
                    }
                }
                other if other.starts_with("wait:") => {
                    flush_text(&mut text_buf, &mut actions);
                    let ms = other[5..].parse::<u64>().unwrap_or(0);
                    actions.push(Action::Wait(ms));
                    if cursor_left.is_some() && cursor_after_action.is_none() {
                        cursor_after_action = last_text_index(&actions);
                    }
                }
                _ => {
                    text_buf.push_str(&format!("{{{{{tag}}}}}"));
                }
            }
            i += consumed;
            continue;
        }
        text_buf.push(chars[i]);
        i += 1;
    }

    flush_text(&mut text_buf, &mut actions);
    if cursor_left.is_some() && cursor_after_action.is_none() {
        cursor_after_action = last_text_index(&actions);
    }

    Plan {
        actions,
        cursor_left,
        cursor_after_action,
    }
}

fn flush_text(buf: &mut String, actions: &mut Vec<Action>) {
    if !buf.is_empty() {
        actions.push(Action::Text(std::mem::take(buf)));
    }
}

fn last_text_index(actions: &[Action]) -> Option<usize> {
    actions.iter().rposition(|a| matches!(a, Action::Text(_)))
}

fn try_tag(chars: &[char]) -> Option<(String, usize)> {
    if chars.len() < 4 || chars[0] != '{' || chars[1] != '{' {
        return None;
    }
    let mut j = 2usize;
    while j + 1 < chars.len() {
        if chars[j] == '}' && chars[j + 1] == '}' {
            let tag: String = chars[2..j].iter().collect::<String>().to_ascii_lowercase();
            return Some((tag, j + 2));
        }
        j += 1;
    }
    None
}

fn text_until_next_control(chars: &[char]) -> String {
    let mut out = String::new();
    let mut i = 0usize;
    while i < chars.len() {
        if let Some((tag, _)) = try_tag(&chars[i..]) {
            if tag == "tab"
                || tag == "enter"
                || tag == "cursor"
                || tag.starts_with("wait:")
            {
                break;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Grapheme clusters; `\r\n` counts as one editor step.
fn grapheme_units(s: &str) -> usize {
    let normalized = s.replace("\r\n", "\n");
    UnicodeSegmentation::graphemes(normalized.as_str(), true).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_left_counts_after_marker() {
        let plan = parse("pre{{cursor}}ab", "");
        assert_eq!(plan.cursor_left, Some(2));
        assert_eq!(plan.actions, vec![Action::Text("preab".into())]);
        assert_eq!(plan.cursor_after_action, Some(0));
    }

    #[test]
    fn tab_splits_actions() {
        let plan = parse("a{{tab}}b{{tab}}c", "");
        assert_eq!(
            plan.actions,
            vec![
                Action::Text("a".into()),
                Action::Tab,
                Action::Text("b".into()),
                Action::Tab,
                Action::Text("c".into()),
            ]
        );
    }

    #[test]
    fn wait_and_enter() {
        let plan = parse("x{{wait:50}}{{enter}}y", "");
        assert_eq!(
            plan.actions,
            vec![
                Action::Text("x".into()),
                Action::Wait(50),
                Action::Enter,
                Action::Text("y".into()),
            ]
        );
    }
}
