import { invoke } from "@tauri-apps/api/core";
import type {
  DeviceStatus,
  InjectionMode,
  PostPasteAction,
  ProfileRule,
  ProfileStatus,
  SnippetSlot,
  TriggerProfileId,
  WheelView,
  WriteVerifyResult,
} from "./types";

/** Rust serde camelCase ↔ TS. */
interface RawSlot {
  layer: number;
  key: number;
  label: string;
  body: string;
  secret: boolean;
  postPaste: string;
  trigger: string;
  injectionMode?: string;
  typingDelayMs?: number;
}

function mapPostPaste(raw: string): PostPasteAction {
  if (raw === "enter" || raw === "Enter") return "enter";
  if (raw === "tab" || raw === "Tab") return "tab";
  return "none";
}

function mapMode(raw: string | undefined): InjectionMode {
  if (raw === "clipboardTerminal" || raw === "ClipboardTerminal") {
    return "clipboardTerminal";
  }
  if (raw === "simulatedTyping" || raw === "SimulatedTyping") {
    return "simulatedTyping";
  }
  return "clipboardPlain";
}

function fromRaw(raw: RawSlot): SnippetSlot {
  return {
    layer: raw.layer,
    key: raw.key,
    label: raw.label,
    body: raw.body,
    secret: raw.secret,
    postPaste: mapPostPaste(raw.postPaste),
    trigger: raw.trigger,
    injectionMode: mapMode(raw.injectionMode),
    typingDelayMs: raw.typingDelayMs ?? 0,
  };
}

function toRaw(slot: SnippetSlot): RawSlot {
  const postPaste =
    slot.postPaste === "enter"
      ? "enter"
      : slot.postPaste === "tab"
        ? "tab"
        : "none";
  return {
    layer: slot.layer,
    key: slot.key,
    label: slot.label,
    body: slot.body,
    secret: slot.secret,
    postPaste,
    trigger: slot.trigger,
    injectionMode: slot.injectionMode,
    typingDelayMs: slot.typingDelayMs,
  };
}

export async function listSlots(): Promise<SnippetSlot[]> {
  const rows = await invoke<RawSlot[]>("list_slots");
  return rows.map(fromRaw);
}

export async function saveSnippet(slot: SnippetSlot): Promise<SnippetSlot> {
  const saved = await invoke<RawSlot>("save_snippet", { slot: toRaw(slot) });
  return fromRaw(saved);
}

export async function osdSaveSlot(slot: SnippetSlot): Promise<SnippetSlot> {
  const saved = await invoke<RawSlot>("osd_save_slot", { slot: toRaw(slot) });
  return fromRaw(saved);
}

export async function registerTrigger(
  layer: number,
  key: number,
  trigger: string,
): Promise<SnippetSlot> {
  const saved = await invoke<RawSlot>("register_trigger", {
    layer,
    key,
    trigger,
  });
  return fromRaw(saved);
}

export async function pasteTest(layer: number, key: number): Promise<void> {
  await invoke("paste_test", { layer, key });
}

export async function getClipboardText(): Promise<string> {
  return invoke<string>("get_clipboard_text");
}

export async function setTiming(opts: {
  restoreDelayMs?: number;
  modifierSettleMs?: number;
  typingDelayMs?: number;
}): Promise<void> {
  await invoke("set_timing", {
    restore_delay_ms: opts.restoreDelayMs ?? null,
    modifier_settle_ms: opts.modifierSettleMs ?? null,
    typing_delay_ms: opts.typingDelayMs ?? null,
  });
}

export async function persistSnippets(): Promise<string> {
  return invoke<string>("persist_snippets");
}

export async function osdToggle(): Promise<boolean> {
  return invoke<boolean>("osd_toggle");
}

export async function osdSetVisible(visible: boolean): Promise<boolean> {
  return invoke<boolean>("osd_set_visible", { visible });
}

export async function osdToggleGhost(): Promise<boolean> {
  return invoke<boolean>("osd_toggle_ghost");
}

export async function osdStatus(): Promise<{ visible: boolean; ghost: boolean }> {
  return invoke("osd_status");
}

export async function hidConnect(): Promise<DeviceStatus> {
  return invoke<DeviceStatus>("hid_connect");
}

export async function hidDisconnect(): Promise<DeviceStatus> {
  return invoke<DeviceStatus>("hid_disconnect");
}

export async function hidStatus(): Promise<DeviceStatus> {
  return invoke<DeviceStatus>("hid_status");
}

export async function hidWriteTrigger(
  layer: number,
  key: number,
  profile: TriggerProfileId,
): Promise<WriteVerifyResult> {
  return invoke<WriteVerifyResult>("hid_write_trigger", {
    layer,
    key,
    profile,
  });
}

export async function hidTriggerAccel(
  profile: TriggerProfileId,
  key: number,
): Promise<string> {
  return invoke<string>("hid_trigger_accel", { profile, key });
}

export async function listProfiles(): Promise<ProfileStatus> {
  return invoke<ProfileStatus>("list_profiles");
}

export async function setProfilePin(locked: boolean): Promise<boolean> {
  return invoke<boolean>("set_profile_pin", { locked });
}

export async function switchAppProfile(
  profileId: string,
): Promise<{ profileId: string; name: string }> {
  return invoke("switch_app_profile", { profile_id: profileId });
}

export async function upsertProfileRule(rule: ProfileRule): Promise<ProfileRule> {
  return invoke<ProfileRule>("upsert_profile_rule", { rule });
}

export async function deleteProfile(profileId: string): Promise<void> {
  await invoke("delete_profile", { profile_id: profileId });
}

export async function clipboardWheelSnapshot(): Promise<WheelView> {
  return invoke<WheelView>("clipboard_wheel_snapshot");
}

export async function clipboardWheelPrev(): Promise<WheelView> {
  return invoke<WheelView>("clipboard_wheel_prev");
}

export async function clipboardWheelNext(): Promise<WheelView> {
  return invoke<WheelView>("clipboard_wheel_next");
}

export async function clipboardWheelPaste(): Promise<WheelView> {
  return invoke<WheelView>("clipboard_wheel_paste");
}

export async function exportEkb(path: string, password: string): Promise<string> {
  return invoke<string>("export_ekb", { path, password });
}

export async function importEkb(path: string, password: string): Promise<number> {
  return invoke<number>("import_ekb", { path, password });
}

export async function exportCleanJson(path: string): Promise<string> {
  return invoke<string>("export_clean_json", { path });
}

export async function importCleanJson(path: string): Promise<number> {
  return invoke<number>("import_clean_json", { path });
}
