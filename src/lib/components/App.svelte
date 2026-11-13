<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Header from "./Header.svelte";
  import KeypadCanvas from "./KeypadCanvas.svelte";
  import SnippetEditor from "./SnippetEditor.svelte";
  import HardwareTab from "./HardwareTab.svelte";
  import ModeSwitch from "./ModeSwitch.svelte";
  import StatusBar from "./StatusBar.svelte";
  import ProfileBar from "./ProfileBar.svelte";
  import KeycapPrintSheet from "./KeycapPrintSheet.svelte";
  import { i18n } from "$lib/i18n/i18n.svelte";
  import {
    deleteProfile,
    exportCleanJson,
    exportEkb,
    hidConnect,
    hidDisconnect,
    hidStatus,
    importCleanJson,
    importEkb,
    listProfiles,
    listSlots,
    osdStatus,
    osdToggle,
    pasteTest,
    setProfilePin,
    switchAppProfile,
    upsertProfileRule,
  } from "$lib/ipc";
  import { session } from "$lib/stores/session.svelte";
  import { formatSyncStatus, saveSnippetAndSyncHid } from "$lib/sync/saveAndSync";
  import {
    emptySlot,
    type CanvasSelection,
    type ProfileRule,
    type ProfileStatus,
    type SnippetSlot,
  } from "$lib/types";

  let slots = $state<SnippetSlot[]>([]);
  let syncing = $state(false);
  let profileStatus = $state<ProfileStatus | null>(null);
  let printOpen = $state(false);

  const selectedKey = $derived(session.selectedKey());
  const current = $derived(
    slots.find((s) => s.layer === session.layer && s.key === selectedKey) ??
      emptySlot(session.layer, selectedKey),
  );
  const charCount = $derived(current.body.length);
  const hwMapping = $derived(session.hardwareFor(session.layer, selectedKey));

  const inspectorTitle = $derived.by(() => {
    if (session.selection.kind === "knob") {
      const a =
        session.selection.action === "ccw"
          ? "↺"
          : session.selection.action === "cw"
            ? "↻"
            : "●";
      return `노브 N${session.selection.knob} ${a}`;
    }
    return `슬롯: Key ${String(selectedKey).padStart(2, "0")} (Layer ${session.layer})`;
  });

  onMount(() => {
    void bootstrap();
    const unsubs: Array<() => void> = [];
    void listen<{ profileId: string; name: string }>("app://profile-switched", (e) => {
      session.statusMessage = `프로필 → ${e.payload.name}`;
      void refreshSlotsAndProfiles();
    }).then((u) => unsubs.push(u));
    void listen<boolean>("app://foreground-elevated", (e) => {
      session.elevatedForeground = e.payload;
    }).then((u) => unsubs.push(u));
    const onKey = (e: KeyboardEvent) => {
      if (e.altKey && !e.ctrlKey && !e.metaKey && ["1", "2", "3"].includes(e.key)) {
        e.preventDefault();
        session.layer = Number(e.key);
        session.selection = { kind: "key", key: 1 };
        return;
      }
      if (e.ctrlKey && !e.altKey && e.key.toLowerCase() === "s") {
        e.preventDefault();
        void syncCurrent();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
      unsubs.forEach((u) => u());
    };
  });

  async function refreshSlotsAndProfiles(): Promise<void> {
    slots = await listSlots();
    try {
      profileStatus = await listProfiles();
      session.elevatedForeground = profileStatus.foreground.elevated;
    } catch {
      /* outside tauri */
    }
  }

  async function bootstrap(): Promise<void> {
    try {
      await refreshSlotsAndProfiles();
      session.daemonActive = true;
      session.statusMessage = "48슬롯 · 프로필/휠 활성";
      try {
        const st = await hidStatus();
        session.connected = st.connected;
        if (st.connected) {
          session.statusMessage = `USB 연결됨${st.product ? ` · ${st.product}` : ""}`;
        }
      } catch {
        session.connected = false;
      }
      try {
        const osd = await osdStatus();
        session.osdEnabled = osd.visible;
        session.osdGhost = osd.ghost;
      } catch {
        /* OSD window may not exist outside tauri */
      }
    } catch (e) {
      session.statusMessage = `로드 실패: ${String(e)}`;
    }
  }

  async function onProfilePin(locked: boolean): Promise<void> {
    try {
      await setProfilePin(locked);
      if (profileStatus) profileStatus = { ...profileStatus, pinLocked: locked };
      session.statusMessage = locked ? "프로필 고정 ON" : "프로필 고정 OFF";
    } catch (e) {
      session.statusMessage = `핀 실패: ${String(e)}`;
    }
  }

  async function onProfileSwitch(id: string): Promise<void> {
    try {
      const p = await switchAppProfile(id);
      await refreshSlotsAndProfiles();
      session.statusMessage = `프로필 → ${p.name}`;
    } catch (e) {
      session.statusMessage = `전환 실패: ${String(e)}`;
    }
  }

  async function onProfileUpsert(rule: ProfileRule): Promise<void> {
    try {
      await upsertProfileRule(rule);
      await refreshSlotsAndProfiles();
      session.statusMessage = `규칙 저장: ${rule.name}`;
    } catch (e) {
      session.statusMessage = `규칙 실패: ${String(e)}`;
    }
  }

  async function onProfileDelete(id: string): Promise<void> {
    try {
      await deleteProfile(id);
      await refreshSlotsAndProfiles();
      session.statusMessage = `프로필 삭제: ${id}`;
    } catch (e) {
      session.statusMessage = `삭제 실패: ${String(e)}`;
    }
  }

  async function runBundle(
    kind: "ekbOut" | "ekbIn" | "jsonOut" | "jsonIn",
    path: string,
    password = "",
  ): Promise<void> {
    if (!path.trim()) {
      session.statusMessage = "경로를 입력하세요";
      return;
    }
    try {
      if (kind === "ekbOut") {
        const out = await exportEkb(path, password);
        session.statusMessage = `.ekb 저장: ${out}`;
      } else if (kind === "ekbIn") {
        const n = await importEkb(path, password);
        await refreshSlotsAndProfiles();
        session.statusMessage = `.ekb 가져옴 (${n} slots)`;
      } else if (kind === "jsonOut") {
        const out = await exportCleanJson(path);
        session.statusMessage = `클린 JSON: ${out}`;
      } else {
        const n = await importCleanJson(path);
        await refreshSlotsAndProfiles();
        session.statusMessage = `JSON 가져옴 (${n} slots)`;
      }
    } catch (e) {
      session.statusMessage = `번들 실패: ${String(e)}`;
    }
  }

  function onSlotChange(next: SnippetSlot): void {
    slots = slots.map((s) =>
      s.layer === next.layer && s.key === next.key ? next : s,
    );
    session.dirty = true;
    session.setMode(next.layer, next.key, "A");
  }

  function applySaved(saved: SnippetSlot): void {
    slots = slots.map((s) =>
      s.layer === saved.layer && s.key === saved.key ? saved : s,
    );
  }

  async function syncCurrent(): Promise<void> {
    if (session.inspectorTab === "hardware" || session.selection.kind === "knob") {
      session.dirty = false;
      session.usbSyncPending = !session.connected;
      session.statusMessage = session.connected
        ? "하드웨어 매핑 로컬 확정 · 커스텀 시퀀스 플래시는 Phase 4 확장"
        : "하드웨어 로컬 저장 · USB 동기화 대기";
      return;
    }

    syncing = true;
    try {
      const outcome = await saveSnippetAndSyncHid(current, session.triggerProfile);
      applySaved(outcome.slot);
      session.dirty = false;
      session.usbSyncPending = outcome.kind === "local_only";
      session.statusMessage = formatSyncStatus(outcome);
      if (outcome.kind === "synced") {
        session.connected = true;
        session.flashPress(outcome.slot.key);
      }
    } catch (e) {
      session.statusMessage = `동기화 실패: ${String(e)}`;
    } finally {
      syncing = false;
    }
  }

  async function toggleConnect(): Promise<void> {
    try {
      if (session.connected) {
        await hidDisconnect();
        session.connected = false;
        session.statusMessage = "USB 해제 · 스니펫 데몬 활성";
        return;
      }
      const st = await hidConnect();
      session.connected = st.connected;
      session.statusMessage = st.connected
        ? `USB 연결됨${st.product ? ` · ${st.product}` : ""}`
        : "연결 실패";
    } catch (e) {
      session.connected = false;
      session.statusMessage = `USB 연결 실패: ${String(e)}`;
    }
  }

  function onSelect(sel: CanvasSelection): void {
    session.selection = sel;
    if (sel.kind === "knob") {
      session.inspectorTab = "hardware";
      return;
    }
    const mode = session.modeFor(session.layer, sel.key);
    session.inspectorTab = mode === "B" ? "hardware" : "snippet";
  }

  function onSwapKeys(a: number, b: number): void {
    const sa = slots.find((s) => s.layer === session.layer && s.key === a);
    const sb = slots.find((s) => s.layer === session.layer && s.key === b);
    const emptyA = emptySlot(session.layer, a);
    const emptyB = emptySlot(session.layer, b);
    const nextA: SnippetSlot = {
      ...(sb ?? emptyB),
      layer: session.layer,
      key: a,
      trigger: (sa ?? emptyA).trigger,
    };
    const nextB: SnippetSlot = {
      ...(sa ?? emptyA),
      layer: session.layer,
      key: b,
      trigger: (sb ?? emptyB).trigger,
    };
    slots = slots.map((s) => {
      if (s.layer !== session.layer) return s;
      if (s.key === a) return nextA;
      if (s.key === b) return nextB;
      return s;
    });
    session.swapHardware(session.layer, a, b);
    session.dirty = true;
    session.statusMessage = `슬롯 교체 · ${String(a).padStart(2, "0")} ↔ ${String(b).padStart(2, "0")}`;
  }

  function onTabChange(tab: "snippet" | "hardware"): void {
    session.inspectorTab = tab;
    if (session.selection.kind === "key") {
      session.setMode(session.layer, selectedKey, tab === "hardware" ? "B" : "A");
    }
  }

  async function toggleOsd(): Promise<void> {
    try {
      const visible = await osdToggle();
      session.osdEnabled = visible;
      session.statusMessage = visible
        ? "OSD ON · Ctrl+Shift+O 숨김 · Ctrl+Shift+G Ghost"
        : "OSD OFF";
    } catch (e) {
      session.statusMessage = `OSD 실패: ${String(e)}`;
    }
  }

  async function runPasteTest(): Promise<void> {
    try {
      if (session.dirty) await syncCurrent();
      session.statusMessage = "3초 후 포커스된 창에 붙여넣기…";
      await new Promise((r) => setTimeout(r, 3000));
      await pasteTest(session.layer, selectedKey);
      session.statusMessage = "paste_test 완료";
      session.flashPress(selectedKey);
    } catch (e) {
      session.statusMessage = `paste_test 실패: ${String(e)}`;
    }
  }
</script>

<main class="flex h-screen w-screen flex-col overflow-hidden bg-bg-canvas text-text-primary">
  <Header
    layer={session.layer}
    connected={session.connected}
    daemonActive={session.daemonActive}
    dirty={session.dirty}
    {syncing}
    triggerProfile={session.triggerProfile}
    osdEnabled={session.osdEnabled}
    onLayerChange={(l) => {
      session.layer = l;
      session.selection = { kind: "key", key: 1 };
    }}
    onProfileChange={(p) => {
      session.triggerProfile = p;
      session.dirty = true;
    }}
    onRotate={() => session.rotateCanvas()}
    onOsdToggle={() => void toggleOsd()}
    onSync={() => void syncCurrent()}
    onConnectToggle={() => void toggleConnect()}
    onPrintLegend={() => (printOpen = true)}
  />

  <ProfileBar
    status={profileStatus}
    onPin={(locked) => void onProfilePin(locked)}
    onSwitch={(id) => void onProfileSwitch(id)}
    onUpsert={(rule) => void onProfileUpsert(rule)}
    onDelete={(id) => void onProfileDelete(id)}
    onExportEkb={(path, password) => void runBundle("ekbOut", path, password)}
    onImportEkb={(path, password) => void runBundle("ekbIn", path, password)}
    onExportClean={(path) => void runBundle("jsonOut", path)}
    onImportClean={(path) => void runBundle("jsonIn", path)}
  />

  <section class="grid min-h-0 flex-1 grid-cols-[420px_1fr] gap-px bg-border-subtle">
    <div class="min-h-0 overflow-hidden bg-bg-panel">
      <KeypadCanvas
        layer={session.layer}
        {slots}
        selection={session.selection}
        onSelect={onSelect}
        onSwapKeys={onSwapKeys}
      />
    </div>
    <div class="flex min-h-0 flex-col overflow-hidden bg-bg-panel">
      <div class="relative">
        <ModeSwitch tab={session.inspectorTab} onChange={onTabChange} />
        <button
          type="button"
          class="absolute right-4 top-3 rounded border border-border-subtle px-2 py-0.5 text-[10px] text-text-secondary hover:border-accent-cyan hover:text-accent-cyan"
          onclick={() => void runPasteTest()}
        >
          {i18n.t("pasteTest")}
        </button>
      </div>
      <div class="min-h-0 flex-1 overflow-hidden">
        {#if session.inspectorTab === "hardware" || session.selection.kind === "knob"}
          <HardwareTab
            mapping={hwMapping}
            title={inspectorTitle}
            onChange={(m) => {
              session.setHardware(m);
              if (session.selection.kind === "key") {
                session.setMode(session.layer, selectedKey, "B");
              }
            }}
          />
        {:else}
          <SnippetEditor slot={current} onChange={onSlotChange} />
        {/if}
      </div>
    </div>
  </section>

  <StatusBar
    message={session.dirty ? `● 미저장 · ${session.statusMessage}` : session.statusMessage}
    {charCount}
    elevatedForeground={session.elevatedForeground}
    usbSyncPending={session.usbSyncPending}
  />

  <KeycapPrintSheet
    open={printOpen}
    {slots}
    modeFor={(layer, key) => session.modeFor(layer, key)}
    onClose={() => (printOpen = false)}
  />
</main>
