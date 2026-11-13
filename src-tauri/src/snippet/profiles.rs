//! App-aware profile rules + slot banks (Zero Flash Wear: host remap only).

use serde::{Deserialize, Serialize};

use super::store::{SnippetSlot, SnippetStore, SLOT_COUNT};

pub const DEFAULT_PROFILE_ID: &str = "default";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MatchRules {
    pub executables: Vec<String>,
    pub title_contains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRule {
    pub profile_id: String,
    pub name: String,
    pub priority: i32,
    pub match_rules: MatchRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppProfile {
    pub rule: ProfileRule,
    pub slots: Vec<SnippetSlot>,
}

impl AppProfile {
    pub fn default_from_slots(slots: Vec<SnippetSlot>) -> Self {
        Self {
            rule: ProfileRule {
                profile_id: DEFAULT_PROFILE_ID.into(),
                name: "Default".into(),
                priority: 0,
                match_rules: MatchRules::default(),
            },
            slots,
        }
    }
}

/// Highest priority among matching rules; None → caller uses Default.
pub fn match_profile<'a>(
    profiles: &'a [AppProfile],
    exe: &str,
    title: &str,
) -> Option<&'a AppProfile> {
    let exe_l = exe.to_ascii_lowercase();
    let title_l = title.to_ascii_lowercase();
    let mut best: Option<&AppProfile> = None;
    for p in profiles {
        if p.rule.profile_id == DEFAULT_PROFILE_ID {
            continue;
        }
        if !rule_matches(&p.rule.match_rules, &exe_l, &title_l) {
            continue;
        }
        match best {
            None => best = Some(p),
            Some(cur) if p.rule.priority > cur.rule.priority => best = Some(p),
            _ => {}
        }
    }
    best
}

fn rule_matches(rules: &MatchRules, exe_l: &str, title_l: &str) -> bool {
    let exe_hit = rules
        .executables
        .iter()
        .any(|e| exe_l == e.to_ascii_lowercase() || exe_l.ends_with(&format!("\\{}", e.to_ascii_lowercase())));
    let title_hit = !rules.title_contains.is_empty()
        && rules
            .title_contains
            .iter()
            .any(|t| title_l.contains(&t.to_ascii_lowercase()));
    if rules.executables.is_empty() && rules.title_contains.is_empty() {
        return false;
    }
    if !rules.executables.is_empty() && !exe_hit {
        return false;
    }
    if !rules.title_contains.is_empty() && !title_hit {
        return false;
    }
    true
}

pub fn ensure_default_profile(store: &mut SnippetStore) {
    if store.profiles.is_empty() {
        store
            .profiles
            .push(AppProfile::default_from_slots(store.slots.clone()));
        store.active_profile_id = DEFAULT_PROFILE_ID.into();
        return;
    }
    if !store
        .profiles
        .iter()
        .any(|p| p.rule.profile_id == DEFAULT_PROFILE_ID)
    {
        store
            .profiles
            .insert(0, AppProfile::default_from_slots(store.slots.clone()));
    }
    if store.slots.len() != SLOT_COUNT {
        store.slots = SnippetStore::default().slots;
    }
}

pub fn sync_active_slots(store: &mut SnippetStore) {
    let id = store.active_profile_id.clone();
    if let Some(p) = store
        .profiles
        .iter_mut()
        .find(|p| p.rule.profile_id == id)
    {
        p.slots = store.slots.clone();
    }
}

/// Remap host snippet set only — never touches pad flash.
pub fn switch_profile(
    store: &mut SnippetStore,
    profile_id: &str,
) -> Result<AppProfile, String> {
    if store.pin_locked && profile_id != store.active_profile_id {
        return Err("profile pinned".into());
    }
    ensure_default_profile(store);
    sync_active_slots(store);
    let profile = store
        .profiles
        .iter()
        .find(|p| p.rule.profile_id == profile_id)
        .cloned()
        .ok_or_else(|| format!("unknown profile: {profile_id}"))?;
    if profile.slots.len() != SLOT_COUNT {
        return Err("profile slot count mismatch".into());
    }
    store.slots = profile.slots.clone();
    store.active_profile_id = profile_id.to_string();
    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_and_default_fallback() {
        let mut store = SnippetStore::default();
        ensure_default_profile(&mut store);
        store.profiles.push(AppProfile {
            rule: ProfileRule {
                profile_id: "hwp".into(),
                name: "한글".into(),
                priority: 10,
                match_rules: MatchRules {
                    executables: vec!["hwp.exe".into()],
                    title_contains: vec![],
                },
            },
            slots: store.slots.clone(),
        });
        assert!(match_profile(&store.profiles, "HWP.EXE", "문서").is_some());
        assert!(match_profile(&store.profiles, "code.exe", "x").is_none());
    }
}
