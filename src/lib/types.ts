export type PostPasteAction = "none" | "enter" | "tab";

export type InjectionMode =
  | "clipboardPlain"
  | "clipboardTerminal"
  | "simulatedTyping";

/** A = snippet (Amber), B = hardware (Cyan). */
export type SlotMode = "A" | "B";

export type TriggerProfileId = "wiredDongle" | "bluetooth";

export type LedColor =
  | "red"
  | "green"
  | "blue"
  | "yellow"
  | "cyan"
  | "purple"
  | "white"
  | "off";

export type CanvasRotation = 0 | 90 | 180 | 270;

export type KnobAction = "ccw" | "press" | "cw";

export interface MatchRules {
  executables: string[];
  titleContains: string[];
}

export interface ProfileRule {
  profileId: string;
  name: string;
  priority: number;
  matchRules: MatchRules;
}

export interface ForegroundInfo {
  exe: string;
  title: string;
  elevated: boolean;
}

export interface ProfileStatus {
  activeProfileId: string;
  pinLocked: boolean;
  profiles: ProfileRule[];
  foreground: ForegroundInfo;
}

export interface WheelView {
  items: string[];
  cursor: number;
  preview: string;
}

export type CanvasSelection =
  | { kind: "key"; key: number }
  | { kind: "knob"; knob: 1 | 2 | 3; action: KnobAction };

export type InspectorTab = "snippet" | "hardware";

export interface SnippetSlot {
  layer: number;
  key: number;
  label: string;
  body: string;
  secret: boolean;
  postPaste: PostPasteAction;
  trigger: string;
  injectionMode: InjectionMode;
  /** 0 = store default (5–20ms). */
  typingDelayMs: number;
}

/** One timeline block: optional modifiers + key, with delay pill after previous. */
export interface HardwareStroke {
  id: string;
  mods: string[];
  key: string;
  delayMs: number;
}

export interface HardwareMapping {
  layer: number;
  key: number;
  strokes: HardwareStroke[];
  scrollSpeed: number;
  mouseDx: number;
  mouseDy: number;
  led: LedColor;
}

export interface DeviceStatus {
  connected: boolean;
  path: string | null;
  interface: number | null;
  usagePage: number | null;
  usage: number | null;
  product: string | null;
}

export interface WriteVerifyResult {
  ok: boolean;
  accel: string;
  writtenCodes: number[];
  readbackCodes: number[];
}

export const LED_SWATCHES: { id: LedColor; hex: string; label: string }[] = [
  { id: "red", hex: "#EF4444", label: "Red" },
  { id: "green", hex: "#22C55E", label: "Green" },
  { id: "blue", hex: "#3B82F6", label: "Blue" },
  { id: "yellow", hex: "#EAB308", label: "Yellow" },
  { id: "cyan", hex: "#06B6D4", label: "Cyan" },
  { id: "purple", hex: "#A855F7", label: "Purple" },
  { id: "white", hex: "#F3F4F6", label: "White" },
  { id: "off", hex: "#262932", label: "Off" },
];

export function emptySlot(layer: number, key: number): SnippetSlot {
  const trigger =
    layer === 1 && key >= 1 && key <= 12 ? `F${12 + key}` : "";
  return {
    layer,
    key,
    label: "",
    body: "",
    secret: false,
    postPaste: "none",
    trigger,
    injectionMode: "clipboardPlain",
    typingDelayMs: 0,
  };
}

export function emptyHardware(layer: number, key: number): HardwareMapping {
  return {
    layer,
    key,
    strokes: [],
    scrollSpeed: 1,
    mouseDx: 0,
    mouseDy: 0,
    led: "off",
  };
}

export function slotKey(layer: number, key: number): string {
  return `${layer}-${key}`;
}

export function ledHex(led: LedColor): string {
  return LED_SWATCHES.find((s) => s.id === led)?.hex ?? "#262932";
}

export function newStrokeId(): string {
  return `s-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
}
