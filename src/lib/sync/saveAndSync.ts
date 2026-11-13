import {
  hidStatus,
  hidWriteTrigger,
  registerTrigger,
  saveSnippet,
} from "$lib/ipc";
import type {
  SnippetSlot,
  TriggerProfileId,
  WriteVerifyResult,
} from "$lib/types";

export type SyncOutcome =
  | {
      kind: "synced";
      slot: SnippetSlot;
      verify: WriteVerifyResult;
    }
  | {
      kind: "local_only";
      slot: SnippetSlot;
      reason: "disconnected" | "write_failed";
      detail?: string;
    };

/** Persist snippet, then if USB pad is open write dual-trigger + register accel. */
export async function saveSnippetAndSyncHid(
  slot: SnippetSlot,
  profile: TriggerProfileId,
): Promise<SyncOutcome> {
  const saved = await saveSnippet(slot);

  let connected = false;
  try {
    const status = await hidStatus();
    connected = status.connected;
  } catch {
    connected = false;
  }

  if (!connected) {
    return { kind: "local_only", slot: saved, reason: "disconnected" };
  }

  try {
    const verify = await hidWriteTrigger(saved.layer, saved.key, profile);
    const withTrigger = await registerTrigger(
      saved.layer,
      saved.key,
      verify.accel,
    );
    return { kind: "synced", slot: withTrigger, verify };
  } catch (e) {
    return {
      kind: "local_only",
      slot: saved,
      reason: "write_failed",
      detail: String(e),
    };
  }
}

export function formatSyncStatus(outcome: SyncOutcome): string {
  if (outcome.kind === "synced") {
    const check = outcome.verify.ok ? "✓ read-back OK" : "⚠ read-back mismatch";
    return `동기화됨 · Key ${String(outcome.slot.key).padStart(2, "0")} → ${outcome.slot.trigger} (${check})`;
  }
  if (outcome.reason === "disconnected") {
    return `로컬 저장 · USB 동기화 대기 (Key ${String(outcome.slot.key).padStart(2, "0")})`;
  }
  return `로컬 저장 · HID 기록 실패: ${outcome.detail ?? "unknown"}`;
}
