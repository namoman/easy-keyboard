import {
  emptyHardware,
  type CanvasRotation,
  type CanvasSelection,
  type HardwareMapping,
  type InspectorTab,
  type SlotMode,
  type TriggerProfileId,
  slotKey,
} from "$lib/types";

/** Shared Phase 3 session state (Svelte 5 runes module). */
class SessionStore {
  layer = $state(1);
  selection = $state<CanvasSelection>({ kind: "key", key: 1 });
  inspectorTab = $state<InspectorTab>("snippet");
  triggerProfile = $state<TriggerProfileId>("wiredDongle");
  connected = $state(false);
  daemonActive = $state(true);
  dirty = $state(false);
  usbSyncPending = $state(false);
  canvasRotation = $state<CanvasRotation>(0);
  osdEnabled = $state(false);
  osdGhost = $state(false);
  /** Physical press feedback (key 1–16). */
  pressedKey = $state<number | null>(null);
  statusMessage = $state("");
  /** Elevated foreground UIPI hint (pitfall 3 stub). */
  elevatedForeground = $state(false);

  /** layer-key → A|B */
  slotModes = $state<Record<string, SlotMode>>({});
  /** layer-key → hardware mapping */
  hardware = $state<Record<string, HardwareMapping>>({});

  selectedKey(): number {
    return this.selection.kind === "key" ? this.selection.key : 1;
  }

  modeFor(layer: number, key: number): SlotMode {
    return this.slotModes[slotKey(layer, key)] ?? "A";
  }

  setMode(layer: number, key: number, mode: SlotMode): void {
    this.slotModes = { ...this.slotModes, [slotKey(layer, key)]: mode };
    this.dirty = true;
  }

  hardwareFor(layer: number, key: number): HardwareMapping {
    const id = slotKey(layer, key);
    return this.hardware[id] ?? emptyHardware(layer, key);
  }

  setHardware(mapping: HardwareMapping): void {
    const id = slotKey(mapping.layer, mapping.key);
    this.hardware = { ...this.hardware, [id]: mapping };
    this.dirty = true;
  }

  swapHardware(layer: number, a: number, b: number): void {
    const ka = slotKey(layer, a);
    const kb = slotKey(layer, b);
    const ha = this.hardware[ka] ?? emptyHardware(layer, a);
    const hb = this.hardware[kb] ?? emptyHardware(layer, b);
    const nextA = { ...hb, layer, key: a };
    const nextB = { ...ha, layer, key: b };
    const modes = { ...this.slotModes };
    const ma = modes[ka] ?? "A";
    const mb = modes[kb] ?? "A";
    modes[ka] = mb;
    modes[kb] = ma;
    this.slotModes = modes;
    this.hardware = { ...this.hardware, [ka]: nextA, [kb]: nextB };
    this.dirty = true;
  }

  rotateCanvas(): void {
    const order: CanvasRotation[] = [0, 90, 180, 270];
    const i = order.indexOf(this.canvasRotation);
    this.canvasRotation = order[(i + 1) % order.length]!;
  }

  flashPress(key: number): void {
    this.pressedKey = key;
    window.setTimeout(() => {
      if (this.pressedKey === key) this.pressedKey = null;
    }, 120);
  }
}

export const session = new SessionStore();
