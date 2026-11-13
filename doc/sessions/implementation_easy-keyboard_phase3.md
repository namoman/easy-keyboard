# Phase 3 implementation — KeypadCanvas + HardwareTab + Header UX

## 범위
UI/UX 스펙 §3–4.1·4.3 기준 KeypadCanvas/HardwareTab/Header 완성, 모드 스위치, 슬롯 모드 스토어, Phase 1 `save_snippet` → Phase 2 `hid_write_trigger` 연동.

## 구현 요약

### KeypadCanvas
- `KnobUnit` ×3 (CCW / press / CW)
- `KeycapTile` 78×78, 모드 점(Amber/Cyan/Rose), LED `border-b-2`, 드래그 스왑, `scale(0.96)` 프레스 피드백
- 캔버스 회전 0/90/180/270 (`↻回転`)

### HardwareTab
- 최대 18타 타임라인 + DelayPill 0–6000ms (클릭/휠)
- 스크롤 1x–5x, 2D 조이스틱(+숫자), 7색(+Off) LED 스와치

### Header / Mode
- USB 연결 토글 + 데몬 배지, 유선/BT 프로필, Layer Alt+1/2/3
- OSD 토글 스텁, Sync(dirty→Amber, Ctrl+S)
- 인스펙터 Snippet(Amber) / Hardware(Cyan) 모드 스위치

### 통합 (`sync/saveAndSync.ts`)
1. `save_snippet`
2. `hid_status` → 연결 시 `hid_write_trigger(profile)` → `register_trigger(accel)`
3. 미연결: 로컬 저장 + StatusBar `USB 대기`

### 스토어
`stores/session.svelte.ts` — layer/selection/mode A|B/hardware map/dirty/connected/rotation/OSD

## 비범위 (의도적)
OSD 창, AES/crypto, `{{cursor}}` 파서, 클립보드 링, 앱 프로필, 커스텀 하드웨어 시퀀스 플래시 write(UI만)

## 검증
- `npm run check` — 0 errors / 0 warnings
- `cargo check` (PATH에 `~/.cargo/bin`) — OK
