# Phase 1 implementation — EasyKeyboard snippet core

## 범위
ClipboardPlain injector + 48슬롯 인메모리 + 트레이 + Header/2단/StatusBar + SnippetEditor + 최소 4×4 KeypadCanvas.

## 구현 요약
- `snippet/injector.rs`: 백업 → `{{date}}`/`{{time}}`/`{{clipboard}}` 치환 → set → Alt/Shift Up + settle → Ctrl+V → restore delay → `IS_INJECTING` 클리어
- `snippet/store.rs`: layer×key 48슬롯, post-paste none/enter/tab, restore/settle ms clamp
- `shortcuts.rs` + `commands.rs`: F13–F24(및 커스텀) 등록, `save_snippet` / `register_trigger` / `paste_test`
- `tray.rs`: 설정 창 show/hide, CloseRequested → hide
- UI: Amber 선택 링, 클립보드 원클릭 empty state, 변수 칩, Secret blur, post-paste 토글
- Phase 2 `hid/`와 `lib.rs`·capabilities를 최소 배선으로 공존 (permissions TOML 분리)

## 비범위 (의도적)
HID write/read-back 검증 UI, OSD, AES/crypto, 영속 DB, parser `{{cursor}}` 등

## 검증
- `npm run check` OK
- `cargo check` OK
