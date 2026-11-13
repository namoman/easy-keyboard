# Phase 2 implementation — HID bridge (`514C:8850`)

## 범위
- `src-tauri/src/hid/` 모듈만 소유 (패킷·듀얼 트리거·디바이스·Tauri 명령)
- SnippetEditor UI / OSD / crypto 미구현
- Macrocli 소스 복사 금지 — MINIKeyboard Web/Win(MIT) + 프로토콜 노트 클린룸

## 파일
| 파일 | 역할 |
|------|------|
| `hid/packet.rs` | `[u8;65]` Report ID `0x03`, commit `03 fd fe ff`, bind/read 빌더 + 단위 테스트 |
| `hid/triggers.rs` | `WiredDongle` (F13–F24 / Ctrl+F13–16) · `Bluetooth` (Ctrl+Alt+Shift+A–P) |
| `hid/k8850.rs` | VID/PID open, vendor usage page 우선(키보드 MI 회피), bind/read/verify |
| `hid/mod.rs` | `HidState` + Tauri commands |
| `permissions/hid.toml` | `allow-hid-bridge` ACL |
| `lib.rs` | Phase 1 모듈 유지 + `HidState` / HID invoke 병합 |

## 함정 4 준수
- 설정 인터페이스: vendor usage page(`≥0xFF00`) 우선, MI_01 선호, plain keyboard(0x01/0x06) 제외
- 모든 write = 정확히 65바이트, Report ID `0x03`
- 커밋 패킷 `03 fd fe ff` + 200ms pause 후 read-back

## Tauri API
- `hid_connect` / `hid_disconnect` / `hid_status`
- `hid_write_trigger(layer, key, profile)` → bind + commit + verify
- `hid_verify_trigger` / `hid_read_layer` / `hid_trigger_accel`

## ACL 함정 (build exit 101)
capabilities에 `allow-hid-connect` 같은 **미정의 문자열** 또는 identifier만 있는 인라인 객체를 넣으면 `Permission … not found`로 build script가 실패한다.  
해결: `src-tauri/permissions/hid.toml`에 `[[permission]]`로 명령을 정의하고, capabilities에서는 `"allow-hid-bridge"` 문자열만 참조.

## 검증
```bash
cd src-tauri && cargo check && cargo test --lib   # 5 passed (packet/triggers)
```
- 실물 패드: USB → `hid_connect` → `hid_write_trigger(1,1,"wiredDongle")` → 물리 키 → Phase 1 F13 스니펫
- 패드 없음: connect 실패는 정상. 단위 테스트로 패킷 규격만 검증

## Phase 1 연동 노트
`save_snippet` 성공 후 동일 `layer`/`key`로 `hid_write_trigger` 호출.  
`profile`: UI 드롭다운 → `"wiredDongle"` | `"bluetooth"`.  
반환 `accel`을 `register_trigger` / 슬롯 `trigger`에 맞춰 global-shortcut과 동기화.
