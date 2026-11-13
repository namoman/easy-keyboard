<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import { listSlots, osdSaveSlot, osdToggleGhost } from "$lib/ipc";
  import type { SnippetSlot, WheelView } from "$lib/types";
  import { emptySlot } from "$lib/types";
  import ClipboardWheelModal from "./ClipboardWheelModal.svelte";

  let layer = $state(1);
  let slots = $state<SnippetSlot[]>([]);
  let ghost = $state(false);
  let pulseKey = $state<number | null>(null);
  let toast = $state("");
  let editKey = $state<number | null>(null);
  let editLabel = $state("");
  let editBody = $state("");
  let profileName = $state("Default");
  let wheelView = $state<WheelView | null>(null);
  let wheelVisible = $state(false);
  let wheelTimer: number | null = null;

  const knobs = [
    { id: "N1", label: "줌" },
    { id: "N2", label: "휠" },
    { id: "N3", label: "볼륨" },
  ];

  const grid = $derived(
    Array.from({ length: 16 }, (_, i) => {
      const key = i + 1;
      const slot =
        slots.find((s) => s.layer === layer && s.key === key) ??
        emptySlot(layer, key);
      return slot;
    }),
  );

  onMount(() => {
    void refresh();
    const unsubs: Array<() => void> = [];
    void listen<{ layer: number; key: number; toast: string }>(
      "osd-pulse",
      (e) => {
        if (e.payload.layer) layer = e.payload.layer;
        pulseKey = e.payload.key;
        toast = e.payload.toast;
        window.setTimeout(() => {
          if (pulseKey === e.payload.key) pulseKey = null;
        }, 150);
        window.setTimeout(() => {
          if (toast === e.payload.toast) toast = "";
        }, 1000);
      },
    ).then((u) => unsubs.push(u));
    void listen<boolean>("osd-ghost", (e) => {
      ghost = e.payload;
    }).then((u) => unsubs.push(u));
    void listen<boolean>("osd-visibility", () => {
      void refresh();
    }).then((u) => unsubs.push(u));
    void listen<{ name: string }>("app://profile-switched", (e) => {
      profileName = e.payload.name || "Default";
      void refresh();
    }).then((u) => unsubs.push(u));
    void listen<WheelView>("app://clipboard-wheel", (e) => {
      wheelView = e.payload;
      wheelVisible = true;
      if (wheelTimer != null) window.clearTimeout(wheelTimer);
      wheelTimer = window.setTimeout(() => {
        wheelVisible = false;
      }, 3000);
    }).then((u) => unsubs.push(u));
    return () => {
      unsubs.forEach((u) => u());
      if (wheelTimer != null) window.clearTimeout(wheelTimer);
    };
  });

  async function refresh(): Promise<void> {
    try {
      slots = await listSlots();
    } catch (e) {
      console.error(e);
    }
  }

  async function toggleGhost(): Promise<void> {
    ghost = await osdToggleGhost();
  }

  async function hide(): Promise<void> {
    await getCurrentWindow().hide();
  }

  function openEdit(slot: SnippetSlot, ev: MouseEvent): void {
    ev.preventDefault();
    if (ghost) return;
    editKey = slot.key;
    editLabel = slot.label;
    editBody = slot.secret ? "" : slot.body;
  }

  async function commitEdit(): Promise<void> {
    if (editKey == null) return;
    const prev =
      slots.find((s) => s.layer === layer && s.key === editKey) ??
      emptySlot(layer, editKey);
    const next: SnippetSlot = {
      ...prev,
      label: editLabel,
      body: editBody.trim() ? editBody : prev.body,
    };
    try {
      const saved = await osdSaveSlot(next);
      slots = slots.map((s) =>
        s.layer === saved.layer && s.key === saved.key ? saved : s,
      );
    } catch (e) {
      console.error(e);
    }
    editKey = null;
  }

  function displayLabel(slot: SnippetSlot): string {
    if (slot.secret) return slot.label || "••••";
    return slot.label || "—";
  }
</script>

<div
  class="relative flex h-full w-full flex-col overflow-hidden rounded-2xl border border-white/10 bg-[#0B0C0E]/80 shadow-2xl backdrop-blur-md transition-opacity duration-150 {ghost
    ? 'opacity-[0.15] outline outline-dashed outline-1 outline-white/30'
    : 'opacity-100'}"
>
  <header
    class="flex h-6 shrink-0 cursor-default items-center justify-between border-b border-white/10 px-2 text-[11px] text-text-secondary"
    data-tauri-drag-region
  >
    <span class="font-mono text-accent-amber">[L{layer}]</span>
    <span class="truncate" title={profileName}>{profileName}</span>
    <div class="flex items-center gap-1">
      <button
        type="button"
        class="rounded px-1 hover:bg-white/10 {ghost ? 'text-accent-cyan' : ''}"
        title="Ghost Mode (Ctrl+Shift+G)"
        onclick={() => void toggleGhost()}
      >
        👻
      </button>
      <button
        type="button"
        class="rounded px-1 hover:bg-white/10"
        title="숨기기"
        onclick={() => void hide()}
      >
        ━
      </button>
      <button
        type="button"
        class="rounded px-1 hover:bg-white/10"
        title="닫기"
        onclick={() => void hide()}
      >
        ×
      </button>
    </div>
  </header>

  {#if toast}
    <div
      class="truncate bg-accent-amber/20 px-2 py-0.5 text-center font-mono text-[10px] text-accent-amber"
    >
      {toast}
    </div>
  {/if}

  <div
    class="flex h-8 shrink-0 items-center justify-around border-b border-white/10 px-1 text-[10px] text-text-secondary"
  >
    {#each knobs as k (k.id)}
      <span class="font-mono">({k.id}: {k.label})</span>
    {/each}
  </div>

  <div class="grid min-h-0 flex-1 grid-cols-4 grid-rows-4 gap-px bg-white/5 p-1">
    {#each grid as slot (slot.key)}
      <button
        type="button"
        class="relative flex flex-col items-center justify-center rounded bg-bg-keycap/80 px-0.5 text-center transition-colors duration-150 {pulseKey ===
        slot.key
          ? 'bg-accent-amber/40 text-accent-amber'
          : 'text-text-primary hover:bg-bg-keycap-hover'}"
        oncontextmenu={(e) => openEdit(slot, e)}
      >
        <span class="absolute left-0.5 top-0.5 font-mono text-[8px] text-text-secondary"
          >{String(slot.key).padStart(2, "0")}</span
        >
        <span class="max-w-full truncate px-1 text-[11px] leading-tight"
          >{displayLabel(slot)}</span
        >
      </button>
    {/each}
  </div>

  <ClipboardWheelModal view={wheelView} visible={wheelVisible} />
</div>

{#if editKey != null}
  <div
    class="absolute inset-0 z-20 flex items-center justify-center bg-black/50 p-3"
  >
    <div
      class="w-full rounded-lg border border-border-subtle bg-bg-panel p-3 shadow-xl"
    >
      <p class="mb-2 font-mono text-[10px] text-accent-amber">
        Key {String(editKey).padStart(2, "0")} 인라인 편집
      </p>
      <input
        class="mb-2 w-full rounded border border-border-subtle bg-bg-canvas px-2 py-1 text-xs outline-none focus:border-accent-amber"
        placeholder="라벨"
        bind:value={editLabel}
      />
      <textarea
        class="mb-2 h-20 w-full resize-none rounded border border-border-subtle bg-bg-canvas p-2 font-mono text-[11px] outline-none focus:border-accent-amber"
        placeholder="스니펫 (Secret이면 비워 두면 유지)"
        bind:value={editBody}
      ></textarea>
      <div class="flex justify-end gap-2">
        <button
          type="button"
          class="rounded border border-border-subtle px-2 py-1 text-[11px]"
          onclick={() => (editKey = null)}>취소</button
        >
        <button
          type="button"
          class="rounded border border-accent-amber bg-accent-amber px-2 py-1 text-[11px] text-[#0B0C0E]"
          onclick={() => void commitEdit()}>저장</button
        >
      </div>
    </div>
  </div>
{/if}
