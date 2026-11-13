# implementation — EasyKeyboard Phase 6

## 목표
키캡 라벨 인쇄 시트(§5.2) + 한/영 UI 크롬 i18n 골격. 제품 로드맵 Phase 0–6 완료.

## 구현 맵

| 영역 | 경로 |
|------|------|
| Print sheet UI | `src/lib/components/KeycapPrintSheet.svelte` |
| Print CSS (mm) | `src/lib/print/keycap-print.css` |
| PNG export 300dpi | `src/lib/print/exportPrintPng.ts` |
| i18n messages | `src/lib/i18n/messages.ts` |
| Locale/theme store | `src/lib/i18n/i18n.svelte.ts` |
| Chrome wiring | `Header` · `StatusBar` · `ProfileBar` · `ModeSwitch` · `App` |

## 인쇄 뷰
- Header **키캡 라벨 인쇄** → 오버레이 미리보기
- 프리셋 `12×12 mm` / `13.5×13.5 mm`
- Layer 1–3 × 4×4, dashed 절취선, 슬롯 번호, 컬러 도트(Amber/Cyan/Rose)
- **인쇄** → `@media print` A4 mm 고정 · **PNG 저장** → canvas data URL (Zero-Network, 외부 라이브러리 없음)

## i18n
- `i18n.setLocale('ko'|'en')` — Header `한|EN` 토글, `localStorage` `ek.locale`
- `i18n.t(key)` — 메인 크롬 문자열
- 테마 훅: `i18n.setTheme('default'|'contrast')` → `html[data-theme]` (UI 토글 없음, API만)

## CSP
`connect-src 'none'` 유지. PNG는 `data:`/`blob:` (`img-src` 허용).

## 잔여 backlog (로드맵 밖)
1. **하드웨어 커스텀 시퀀스 플래시 write** — HardwareTab 로컬 확정만, 65바이트 시퀀스 HID write 미완
2. **노브→휠 HID 매핑 폴리시** — 호스트 단축키(`Ctrl+Alt+Shift+PgUp/Dn/Enter`)는 동작; 패드 플래시에 노브 전용 코드 자동 기록은 스텁/수동

## 검증
```bash
npm run check
cd src-tauri && cargo check && cargo test --lib
```
