# implementation — EasyKeyboard Phase 5

## 목표
앱 인식 프로필 자동전환, 클립보드 히스토리 휠, `.ekb`/클린 JSON 번들.

## 구현 맵

| 영역 | 경로 |
|------|------|
| Watcher | `src-tauri/src/context/window_watcher.rs` |
| Profile rules | `snippet/profiles.rs` (+ store 필드) |
| Clipboard ring | `snippet/clipboard_ring.rs` |
| Exclude clipboard | `snippet/clipboard_set.rs` |
| Bundle | `snippet/bundle.rs` |
| IPC | `commands_ext.rs` + `permissions/snippet.toml` |
| UI | `ProfileBar.svelte`, `ClipboardWheelModal.svelte` |
| OSD | `OsdOverlay.svelte` (profile name + wheel 3s) |

## APIs (Tauri commands)

| Command | 역할 |
|---------|------|
| `list_profiles` | active id · pin · rules · foreground |
| `set_profile_pin(locked)` | 자동전환 잠금 |
| `switch_app_profile(profile_id)` | 수동 전환 (핀 일시 해제) |
| `upsert_profile_rule(rule)` | 규칙 추가/수정 (신규면 현재 슬롯 복사) |
| `delete_profile(profile_id)` | Default 삭제 불가 |
| `clipboard_wheel_{snapshot,prev,next,paste}` | 링 조회/회전/붙여넣기 |
| `export_ekb` / `import_ekb` | Argon2id+AES `.ekb` |
| `export_clean_json` / `import_clean_json` | Secret 비운 JSON |

## 이벤트
- `app://profile-switched` `{ profileId, name, exe, title }`
- `app://clipboard-wheel` `WheelView`
- `app://foreground-elevated` `bool` (UIPI 배지)

## 함정 준수
| # | 반영 |
|---|------|
| 3 | elevated foreground → StatusBar `권한 불일치` |
| 6 | ring은 `is_injecting()` skip · inject `set_text_excluded` |

## Zero Flash Wear
프로필 전환 시 HID write 호출 없음. 동일 F13–F24 트리거가 활성 프로필 슬롯 body만 참조.

## 휠 단축키 (노브 매핑용)
- CCW: `CommandOrControl+Alt+Shift+PageUp`
- CW: `CommandOrControl+Alt+Shift+PageDown`
- Press: `CommandOrControl+Alt+Shift+Enter`

## 검증
`npm run check` · `cargo test --lib` → 12 passed (profiles + ring + ekb 추가)

## Phase 6 잔여
KeycapPrintSheet · i18n · 하드웨어 커스텀 시퀀스 플래시 write
