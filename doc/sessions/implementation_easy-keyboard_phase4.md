# implementation — EasyKeyboard Phase 4

## 목표
OSD 오버레이 창, 주입 모드 라우팅, 템플릿 파서, AES 암호화 영속화, CI/release 골격.

## 구현 맵

| 영역 | 경로 |
|------|------|
| Parser | `src-tauri/src/snippet/parser.rs` |
| Inject routing | `snippet/injector.rs` + `keys.rs` + `unicode_type.rs` |
| Crypto/persist | `snippet/crypto.rs`, `snippet/persist.rs` |
| OSD host | `src-tauri/src/osd.rs` |
| OSD UI | `src/lib/components/OsdOverlay.svelte`, `src/routes/osd/+page.svelte` |
| ACL | `permissions/snippet.toml`, `capabilities/osd-window.json` |
| CI | `.github/workflows/ci.yml`, `release.yml` |
| Guides | `doc/guides/security_crypto.md`, `release_ci.md` |

## 함정 준수
| # | 반영 |
|---|------|
| 1 | paste 전 30–50ms + Alt/Shift Up |
| 2 | 복원 전 80–120ms |
| 5 | SimulatedTyping → `enigo.text` UNICODE (VK 한글 금지) |
| 6 | `IS_INJECTING` SeqCst + 복원 후 50ms clear |

## OSD / Ghost
- 생성: setup 시 hidden `osd` webview (`/osd`)
- 표시: `osd_toggle` / `Ctrl+Shift+O`
- Ghost: `set_ignore_cursor_events` + 프론트 `opacity-[0.15]` / `Ctrl+Shift+G`

## 암호문 위치
`app_data_dir()/snippets.enc` — 보통  
`%APPDATA%\com.namoman.easy-keyboard\snippets.enc`

## 검증
`npm run check` · `cargo check` · `cargo test --lib` (parser 3 + crypto 1 + hid 5)

## Phase 5 훅
- `IS_INJECTING` / `is_injecting()` — clipboard_ring
- `app.emit("osd-pulse"|"osd-visibility")` — profile switch 라벨 갱신
- store serde 스키마 — `.ekb` bundle 입력
