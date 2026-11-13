/** ko/en chrome strings — Phase 6 skeleton (UI chrome only). */

export type LocaleId = "ko" | "en";

export type MessageKey =
  | "appName"
  | "usbConnected"
  | "daemonActive"
  | "idle"
  | "profileWired"
  | "profileBt"
  | "layer"
  | "rotate"
  | "osd"
  | "sync"
  | "syncing"
  | "printLegend"
  | "langKo"
  | "langEn"
  | "profile"
  | "pinOn"
  | "pinOff"
  | "pinTitle"
  | "addRule"
  | "ekb"
  | "cancel"
  | "save"
  | "namePlaceholder"
  | "exePlaceholder"
  | "titlePlaceholder"
  | "pathPlaceholder"
  | "passwordPlaceholder"
  | "exportEkb"
  | "importEkb"
  | "cleanJson"
  | "importJson"
  | "tabSnippet"
  | "tabHardware"
  | "pasteTest"
  | "elevatedMismatch"
  | "usbPending"
  | "charCount"
  | "printTitle"
  | "printPreset12"
  | "printPreset135"
  | "printLayers"
  | "printDo"
  | "printPng"
  | "printClose"
  | "printHint"
  | "themeDefault"
  | "themeContrast";

type Dictionary = Record<MessageKey, string>;

const ko: Dictionary = {
  appName: "EasyKeyboard",
  usbConnected: "USB 연결됨",
  daemonActive: "스니펫 데몬 활성",
  idle: "대기",
  profileWired: "동글/유선 (F13–F24)",
  profileBt: "블루투스 (조합키)",
  layer: "Layer",
  rotate: "↻回転",
  osd: "OSD",
  sync: "기기에 동기화",
  syncing: "동기화…",
  printLegend: "키캡 라벨 인쇄",
  langKo: "한",
  langEn: "EN",
  profile: "프로필",
  pinOn: "📌 고정",
  pinOff: "📌",
  pinTitle: "현재 프로필 고정",
  addRule: "+규칙",
  ekb: ".ekb",
  cancel: "취소",
  save: "저장",
  namePlaceholder: "이름",
  exePlaceholder: "exe (쉼표 구분) e.g. hwp.exe",
  titlePlaceholder: "창 제목 포함 (쉼표)",
  pathPlaceholder: "경로 e.g. D:\\backup\\profile.ekb",
  passwordPlaceholder: "마스터 비밀번호 (.ekb)",
  exportEkb: ".ekb 내보내기",
  importEkb: ".ekb 가져오기",
  cleanJson: "클린 JSON",
  importJson: "JSON 가져오기",
  tabSnippet: "텍스트 스니펫",
  tabHardware: "하드웨어",
  pasteTest: "붙여넣기 테스트 (3초)",
  elevatedMismatch: "권한 불일치",
  usbPending: "USB 대기",
  charCount: "글자 수",
  printTitle: "키캡 라벨지",
  printPreset12: "12 × 12 mm",
  printPreset135: "13.5 × 13.5 mm",
  printLayers: "레이어 1–3",
  printDo: "인쇄",
  printPng: "PNG 저장",
  printClose: "닫기",
  printHint: "절취선에 맞춰 오리고 투명 키캡에 삽입하세요.",
  themeDefault: "기본",
  themeContrast: "고대비",
};

const en: Dictionary = {
  appName: "EasyKeyboard",
  usbConnected: "USB connected",
  daemonActive: "Snippet daemon active",
  idle: "Idle",
  profileWired: "Dongle/wired (F13–F24)",
  profileBt: "Bluetooth (chords)",
  layer: "Layer",
  rotate: "↻ Rotate",
  osd: "OSD",
  sync: "Sync to device",
  syncing: "Syncing…",
  printLegend: "Print keycap legends",
  langKo: "한",
  langEn: "EN",
  profile: "Profile",
  pinOn: "📌 Pin",
  pinOff: "📌",
  pinTitle: "Pin current profile",
  addRule: "+ Rule",
  ekb: ".ekb",
  cancel: "Cancel",
  save: "Save",
  namePlaceholder: "Name",
  exePlaceholder: "exe (comma-separated) e.g. hwp.exe",
  titlePlaceholder: "Title contains (comma)",
  pathPlaceholder: "Path e.g. D:\\backup\\profile.ekb",
  passwordPlaceholder: "Master password (.ekb)",
  exportEkb: "Export .ekb",
  importEkb: "Import .ekb",
  cleanJson: "Clean JSON",
  importJson: "Import JSON",
  tabSnippet: "Text snippet",
  tabHardware: "Hardware",
  pasteTest: "Paste test (3s)",
  elevatedMismatch: "Elevation mismatch",
  usbPending: "USB pending",
  charCount: "Chars",
  printTitle: "Keycap legend sheet",
  printPreset12: "12 × 12 mm",
  printPreset135: "13.5 × 13.5 mm",
  printLayers: "Layers 1–3",
  printDo: "Print",
  printPng: "Save PNG",
  printClose: "Close",
  printHint: "Cut on dashed lines and insert into relegendable caps.",
  themeDefault: "Default",
  themeContrast: "High contrast",
};

export const MESSAGES: Record<LocaleId, Dictionary> = { ko, en };
