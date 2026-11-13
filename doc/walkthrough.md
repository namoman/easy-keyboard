# EasyKeyboard walkthrough

## 2026-11-14 — Phase 6 keycap print / i18n

### 배경
고급 스펙 §5.2 + README P3. Less is more: 외부 CDN/html2canvas 없이 canvas PNG + `@media print` mm CSS. i18n은 메인 크롬만, 테마는 `data-theme` 훅만.

### 변경 요약
- **KeycapPrintSheet**: 12×12 / 13.5×13.5 mm, Layer 1–3 4×4, dashed 절취선·슬롯 번호·Amber/Cyan/Rose 도트
- **진입**: Header「키캡 라벨 인쇄」→ 미리보기 → 인쇄 또는 PNG 저장
- **i18n**: `messages.ts` ko/en · Header `한|EN` · ProfileBar/StatusBar/ModeSwitch/공통 버튼
- **테마 훅**: `i18n.setTheme('contrast')` → `html[data-theme=contrast]` (UI 토글 없음)
- **로드맵**: Phase 0–6 완료. 잔여 = 하드웨어 시퀀스 플래시 write · 노브 HID→휠 매핑 폴리시

### 확인
```bash
npm run check
cd src-tauri && cargo check && cargo test --lib
```

---

## 2026-11-14 — Phase 5 profiles / clipboard wheel / .ekb

### 배경
고급 스펙 §1·§2·§5.1. Phase 4 훅(`IS_INJECTING`, store serde, OSD emit)을 이어 앱 인식 프로필·클립보드 링·번들을 붙였다. Less is more: 키캡 인쇄·i18n·하드웨어 시퀀스 플래시는 Phase 6.

### 변경 요약
- **프로필**: `window_watcher` 150ms foreground → `ProfileRule` 우선순위 → Default 폴백 · Pin 고정 · **Zero Flash Wear**(슬롯 뱅크만 교체)
- **이벤트**: `app://profile-switched` → OSD 라벨 갱신 · `app://foreground-elevated` → StatusBar UIPI 배지
- **클립보드 휠**: RAM 20개 · `IS_INJECTING` 무시 · ExcludeClipboard… · 노브 전용 `Ctrl+Alt+Shift+PgUp/Dn/Enter` · OSD 캐러셀 3초
- **번들**: `.ekb` Argon2id+AES-GCM · 클린 JSON(Secret body 비움) · `ProfileBar` 경로+비밀번호 UI

### 확인
```bash
npm run check                    # 0 errors
cd src-tauri && cargo check && cargo test --lib   # 12 passed
```

### 다음
Phase 6 — KeycapPrintSheet · 한/영 리소스.

---

## 2026-11-14 — Phase 4 OSD / inject / parser / AES / CI

### 배경
Phase 3에서 스텁이던 OSD를 두 번째 Tauri 창으로 교체하고, 주입 3종·템플릿 파서·AES 저장·CI를 붙였다. Less is more: 앱 프로필·클립보드 링·`.ekb`·하드웨어 시퀀스 플래시는 Phase 5로 유지.

### 변경 요약
- **OSD** `260×310`: glassmorphism, 4×4 라벨, 150ms pulse+토스트, 우클릭 인라인 편집, Ghost(무시 커서+15% opacity)
- **토글**: Header OSD / `Ctrl+Shift+O`, Ghost `Ctrl+Shift+G`
- **주입**: ClipboardPlain · ClipboardTerminal(Shift+Insert) · SimulatedTyping(UNICODE, 5–20ms)
- **parser**: `{{cursor}}` Left 역산 · `{{tab}}` · `{{enter}}` · `{{wait:ms}}` (+ date/time/clipboard)
- **crypto**: AES-256-GCM + keyring → `snippets.enc`; Secret은 OSD 라벨만 + 주입 버퍼 zeroize
- **CI**: `.github/workflows/ci.yml`, `release.yml` (Windows + SHA-256 + attest)

### 확인
```bash
npm run check                    # 0 errors
cd src-tauri && cargo check && cargo test --lib   # 9 passed
```

### 다음
Phase 5 — window_watcher 프로필 · clipboard_ring · `.ekb` bundle.

---

## 2026-11-14 — Phase 3 KeypadCanvas / HardwareTab / Sync

### 배경
UI/UX 스펙 §3–4.1·4.3에 맞춰 좌측 키패드 미러·하드웨어 인스펙터·헤더 동기화를 완성하고, Phase 1 스니펫 저장을 Phase 2 `hid_write_trigger`에 연결했다. Less is more: 하드웨어 커스텀 시퀀스 플래시 write·OSD 창·UIPI 실감지는 Phase 4로 남기고 UI/로컬 상태만 고정했다.

### 변경 요약
- **KeypadCanvas**: 노브 3종, 78×78 키캡, Amber/Cyan/Rose 모드 점, LED `border-b-2`, 드래그 스왑, 회전 0–270°, 프레스 `scale(0.96)`
- **HardwareTab**: 18타 타임라인·딜레이 pill(0–6000ms)·스크롤 1–5x·2D 조이스틱·LED 스와치
- **Header**: USB/데몬, 유선|BT, Layer Alt+1/2/3, OSD 스텁, Sync(dirty Amber, Ctrl+S)
- **연동**: `saveAndSync.ts` — 연결 시 write+accel 등록, 미연결 시 로컬+`USB 대기`
- **스토어**: `session.svelte.ts` (mode A|B, hardware map, dirty, connected)

### 확인
```bash
npm run check          # 0 errors
cd src-tauri && cargo check
```

### 다음
Phase 4 — OSD 창, 주입 3종, parser, AES, 하드웨어 시퀀스 HID write.

---

## 2026-11-14 — Phase 2 HID 브릿지 + ACL 수정

### 배경
`514C:8850` HID 브릿지(듀얼 트리거)를 추가했다. 초기 capabilities에 존재하지 않는 `allow-hid-*` 문자열을 넣어 build가 실패했고, `permissions/*.toml`로 ACL을 정식 정의한 뒤 `cargo check`/`test`를 통과시켰다. Phase 1 snippet/UI 파일은 덮어쓰지 않았다.

### 변경 요약
- `hid/packet.rs` · `triggers.rs` · `k8850.rs` · `mod.rs` — 65바이트/`0x03`/commit/`MI` 선택(함정 4)
- `permissions/hid.toml` (`allow-hid-bridge`), `permissions/snippet.toml` (`allow-snippet-core`)
- `capabilities/main-window.json` — 문자열 권한만 참조(인라인 가짜 identifier 제거)
- `lib.rs` — Phase 1 invoke 유지 + HID commands / `HidState`

### 확인
```bash
cd src-tauri && cargo check && cargo test --lib   # 5 passed
```

### 다음
`save_snippet` → `hid_write_trigger` 자동 연동, Header `hid_status` 배지.

---

## 2026-11-14 — Phase 1 스니펫 코어 + UI 골격

### 배경
계획 Phase 1에 따라 ClipboardPlain 주입 엔진과 960×640 설정 UI 골격을 추가했다. HID 장치 write·OSD·암호화는 제외. 동시 진행 중이던 Phase 2 `hid/` 모듈은 `lib.rs`에 병행 등록만 유지했다.

### 변경 요약 (Less is more / 함정 반영)
- **Injector**: 수정자 잔류 방지(Alt/Shift Up + 30–50ms), 클립보드 복원 80–120ms, `IS_INJECTING` SeqCst+50ms 클리어 (`doc/guides/rust_tauri_pitfalls.md` 1·2·6).
- **48슬롯 인메모리** + Layer1 Key01–12 → F13–F24 글로벌 단축키.
- **트레이** show/hide, 창 닫기 시 숨김.
- **UI**: Header 52px / KeypadCanvas 4×4 amber ring / SnippetEditor(원클릭 클립보드·칩·Secret blur·post-paste) / StatusBar 28px.
- **IPC**: `list_slots`, `save_snippet`, `register_trigger`, `paste_test`, `get_clipboard_text`, `set_timing`.
- ACL: `permissions/snippet.toml` (+ Phase 2용 `hid.toml` 공존).

### 확인 방법
```bash
npm run check
cd src-tauri && cargo check   # PATH에 ~/.cargo/bin 필요
npm run tauri dev
# 1) Key 01에 텍스트 저장  2) Notepad 포커스  3) F13 또는 UI「붙여넣기 테스트(3초)」
```

### 다음
Phase 2 HID 실기 마감 후 Phase 3 UI/하드웨어 탭 폴리시, 또는 Phase 4 OSD·주입 분기.

---

## 2026-11-14 — Phase 0 기반 세팅

### 배경
구현 계획 Phase 0에 따라 워크스페이스 루트에 Tauri v2 + Svelte TypeScript 앱을 스캐폴딩하고, 스펙·README·토큰·보안 CSP·capabilities 골격을 고정했다. 제품 기능(스니펫/HID/OSD)은 구현하지 않았다.

### 변경 요약
- `ref/MINIKeyboardWeb` 클론 (기존 Win/Macrocli/ch57x 유지). `ref/`는 `.gitignore`.
- `create-tauri-app` svelte-ts 템플릿을 임시 폴더에 생성 후 루트로 이동, 패키지명 `easy-keyboard`로 정리.
- npm: `@tauri-apps/plugin-global-shortcut`, Tailwind v4 (`@tailwindcss/vite`).
- Cargo: `tauri-plugin-global-shortcut`, `hidapi`, `arboard`, `enigo`, `aes-gcm`, `keyring`, `zeroize`, `serde_json`.
- 스펙 3종 → `doc/spec/`. README는 google/suspicious 링크를 `LICENSE`, `doc/spec/...`, GitHub Releases로 교체.
- MIT `LICENSE`, `THIRD_PARTY_NOTICES.md`, `doc/HANDOFF.md`, `doc/walkthrough.md`, 빈 `doc/guides/`·`doc/sessions/`.
- UI UX 스펙 토큰을 `src/app.css` `@theme` + CSS 변수로 반영. 메인 창 960×640, CSP `connect-src 'none'`, capabilities `main-window` / `osd-window`.

### 확인 방법 / 검증 결과 (2026-11-14)
```bash
npm install          # OK
npm run check        # OK (0 errors)
npm run build        # OK → build/
cd src-tauri && cargo check && cargo build  # OK
# npm run tauri dev / tauri build — Phase 0에서 전체 번들 미실행 (스캐폴드 검증만)
```

### 다음
Phase 1 — `ClipboardPlain` injector + UI 골격.

## 2026-11-14 — Rust/Tauri 함정 가이드 추가

- `doc/guides/rust_tauri_pitfalls.md`에 사용자 확인 6대 함정(수정자 잔류, 클립보드 복원 레이스, UIPI, HID MI_01/65바이트, 한글 UNICODE, 클립보드 링 자기 트리거)을 문제/해결로 문서화하고, `doc/HANDOFF.md`에 Phase 매핑 링크를 넣었다. 제품 기능 코드는 변경하지 않았다.

## 2026-11-14 — Phase 2 HID bridge (`514C:8850`)

### 배경
모드 B 스니펫용 트리거를 패드 플래시에 기록하려면 올바른 config HID 인터페이스와 65바이트 프로토콜이 필요하다. Macrocli는 클린룸(프로토콜 아이디어만), 구현 참고는 MINIKeyboard Web/Win(MIT).

### 변경 요약
- `hid/packet.rs`: Report ID `0x03`, commit `03 fd fe ff`, bind/read 빌더 + 단위 테스트
- `hid/triggers.rs`: `wiredDongle` = F13–F24(+Ctrl+F13–16), `bluetooth` = Ctrl+Alt+Shift+A–P
- `hid/k8850.rs`: vendor usage page 우선 open(키보드 MI 회피), bind_key / read_layer / verify
- Tauri: `hid_connect|disconnect|status|write_trigger|verify_trigger|read_layer|trigger_accel`
- Phase 1 `snippet/`·commands는 유지한 채 `lib.rs`에 `HidState`만 병합
- ACL: `permissions/hid.toml` → `allow-hid-bridge`

### 확인
```bash
cd src-tauri && cargo test --lib   # 5 passed
```
실물: USB 패드 → `hid_connect` → `hid_write_trigger(1, 1, "wiredDongle")` → F13 스니펫.

### Phase 1 연동 메모
`save_snippet` 성공 후 활성 `TriggerProfile`로 `hid_write_trigger(slot.layer, slot.key, profile)` 호출하고, 반환 `accel`을 `register_trigger` / 슬롯 `trigger`에 맞추면 된다.
