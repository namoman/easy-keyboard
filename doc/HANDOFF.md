# EasyKeyboard HANDOFF

## 현재 단계
**Phase 6 완료** — 키캡 라벨 인쇄 · ko/en i18n 골격  
**제품 로드맵 Phase 0–6 전부 완료**

## Frozen State
- 패키지명: `easy-keyboard` (identifier `com.namoman.easy-keyboard`)
- 스택: Tauri v2 + SvelteKit(Svelte 5) + TypeScript + Tailwind v4
- 메인 창: `960×640`, label `main` · OSD: `260×310`, label `osd`, transparent
- CSP: `connect-src 'none'` · capabilities: `main-window` / `osd-window`
- 암호문: `%APPDATA%\com.namoman.easy-keyboard\snippets.enc` + keyring AES 키
- 프로필: Zero Flash Wear · 클립보드 링 RAM 20 · 휠 단축키 `Ctrl+Alt+Shift+PageUp/PageDown/Enter`
- i18n: `ek.locale` (`ko`|`en`) · 테마 훅 `ek.theme` (`default`|`contrast`) → `html[data-theme]`
- Print: 12 / 13.5 mm · `@media print` + 300dpi PNG canvas

## 완료 파일 (Phase 6)
- `KeycapPrintSheet.svelte` + `print/keycap-print.css` + `print/exportPrintPng.ts`
- `i18n/messages.ts` + `i18n/i18n.svelte.ts`
- Header 인쇄 진입 · `한|EN` 토글 · ProfileBar/StatusBar/ModeSwitch 문자열

## 잔여 backlog (로드맵 밖)
1. 하드웨어 커스텀 시퀀스 **플래시 write** (HardwareTab → 65B HID)
2. 노브→클립보드 휠 **패드 HID 자동 매핑** 폴리시 (호스트 단축키는 완료, 플래시 기록은 스텁/수동)

## 검증
```bash
npm run check
cd src-tauri && cargo check && cargo test --lib
```

## Resume 문구
```
EasyKeyboard Phase 0–6 제품 로드맵 완료.
다음(백로그): 하드웨어 커스텀 시퀀스 플래시 write · 노브→휠 HID 매핑 폴리시.
인쇄: Header「키캡 라벨 인쇄」→ 프리셋 → 인쇄/PNG.
i18n: Header 한|EN (localStorage ek.locale).
```
