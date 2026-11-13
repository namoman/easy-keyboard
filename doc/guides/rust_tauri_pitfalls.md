# EasyKeyboard — Rust / Tauri 필수 함정 가이드

> **목적**: 후속 Phase 에이전트·구현자가 이미 확인된 Windows/HID/클립보드 함정을 다시 밟지 않도록 고정한다.  
> **범위**: 제품 기능 구현이 아니라 **구현 시 반드시 지킬 제약**만 기술한다.  
> **관련 Phase**: 아래 각 절의 「적용 Phase」를 따른다.

---

## 1. Modifier key residual collision (수정자 키 잔류 충돌)

**적용 Phase**: Phase 1 — ClipboardPlain injector

### 문제

블루투스(BT) 매크로 콤보로 슬롯이 트리거된 직후 `Ctrl+V`를 보내면, 직전에 눌려 있던 **Alt / Shift** 등이 OS 입력 큐에 아직 남아 있을 수 있다.  
그 결과 실제로는 `Ctrl+Alt+V`, `Ctrl+Shift+V`처럼 **의도하지 않은 단축키**가 실행된다.

- 물리/HID 콤보 → 앱 트리거 → 즉시 `Ctrl+V` 주입 경로에서 특히 자주 발생한다.
- “가끔만” 재현되므로 타이밍 이슈로 오인하기 쉽다.

### 해결

1. **짧은 대기**: `Ctrl+V` 직전에 **30–50ms** 대기한다 (`tokio::time::sleep` 등).
2. **명시적 Key::Up**: 대기만으로 부족하면, paste 전에 **Alt / Shift에 대해 `Key::Up`을 명시적으로 전송**해 잔류 수정자를 클리어한다.
3. 그 다음에야 `Ctrl+V`(Down → Up)를 보낸다.

권장 순서:

```text
(트리거 수신)
→ sleep 30–50ms
→ (선택) Alt Up, Shift Up
→ Ctrl Down → V Down → V Up → Ctrl Up
```

---

## 2. Clipboard restore race (클립보드 복원 레이스)

**적용 Phase**: Phase 1 — ClipboardPlain injector

### 문제

주입 흐름은 대략 다음과 같다.

1. 현재 클립보드 백업
2. 스니펫 텍스트를 클립보드에 기록
3. `Ctrl+V`로 붙여넣기
4. 백업본으로 클립보드 복원

4번을 **붙여넣기 직후 즉시** 하면, 대상 앱이 아직 클립보드를 읽기 전에 내용이 되돌아가 **빈 붙여넣기 / 잘못된 내용**이 된다.  
백업본이 이미 바뀌었거나, Win+V 히스토리와 경합하는 경우도 있다.

### 해결

1. `Ctrl+V` 전송 **이후**에만 복원한다.
2. 복원 전 **`tokio::time::sleep`으로 80–120ms** 대기한다.
3. 이 지연은 나중에 **설정 UI에서 사용자 조절 가능**하게 둘 여지를 남긴다 (기본값 80–120ms 범위).

권장 순서:

```text
backup clipboard
→ set snippet
→ Ctrl+V
→ sleep 80–120ms  (설정값으로 승격 예정)
→ restore backup
```

함정 6(`IS_INJECTING` 가드)과 **반드시 함께** 적용한다.

---

## 3. UIPI / elevated window (권한 상승 창 차단)

**적용 Phase**: 설정 / UI (배지·옵션), injector 동작 검증과 연동

### 문제

Windows **UIPI**(User Interface Privilege Isolation) 때문에, **일반 권한** 프로세스는 **관리자(elevated) 창**으로 키보드/마우스 입력을 주입할 수 없다.  
관리자 터미널·설치 마법사·일부 시스템 UI에 붙여넣기가 **조용히 실패**한다.

### 해결

1. **기본 정책**: 앱은 **일반 권한(non-elevated)** 으로 실행한다. (항상 관리자 실행을 기본으로 하지 않음)
2. **선택적 elevated 호환** (설정 옵션):
   - Task Scheduler로 elevated 작업 등록, 또는
   - 사용자 동의 후 elevated로 재실행(relaunch)
3. **UI 배지**: 활성 창이 elevated일 때 StatusBar 등에 **권한 불일치/제한** 배지를 표시해, “붙여넣기가 안 되는 이유”를 사용자가 바로 알 수 있게 한다.

구현 시 주의:

- elevated를 강제하지 않는다.
- 배지는 “현재 foreground가 elevated인가” 감지에 기반한다.

---

## 4. HID interface MI_01 vs MI_00 + 65-byte padding

**적용 Phase**: Phase 2 — HID 연동

### 문제

대상 장치 **VID:PID `514C:8850`** 은 복합 HID일 수 있다.

- 잘못된 인터페이스(예: **MI_00**)를 열면 open은 성공해도 **리포트가 무시**되거나 매크로/LED 경로가 동작하지 않는다.
- 올바른 인터페이스는 보통 **MI_01**(해당 usage page/usage)이다.
- Windows에서 feature/output 리포트는 **Report ID 포함 고정 길이**가 필요하다. 길이가 짧거나 Report ID가 빠지면 write가 실패하거나 no-op이 된다.

### 해결

1. **올바른 usage page / interface**로 open한다 (`514C:8850`, 매크로 경로에 해당하는 MI — 통상 **MI_01**).
2. write 버퍼는 **정확히 `[u8; 65]`** (Report ID 1바이트 + 64바이트 페이로드).
3. Report ID는 **`0x03`**.
4. 커밋(적용) 패킷 예:

```text
03 fd fe ff  (+ 나머지 0 패딩으로 총 65바이트)
```

즉 선두는 `0x03, 0xfd, 0xfe, 0xff` 이고, 나머지는 `0`으로 채워 **길이 65**를 맞춘다.

체크리스트:

- [ ] 장치 열 때 interface/usage 필터 확인
- [ ] 모든 HID write가 `[u8; 65]`
- [ ] Report ID `0x03` 누락 없음
- [ ] 커밋 `03 fd fe ff` 전송 후 장치 반영 확인

---

## 5. SimulatedTyping Hangul (한글 시뮬 타이핑)

**적용 Phase**: Phase 4 — SimulatedTyping

### 문제

VK(가상 키) 매핑이나 스캔코드 기반 타이핑으로 한글을 보내면:

- IME 상태·레이아웃에 따라 **깨진 문자 / 영문 대체 / 조합 실패**가 난다.
- “한 글자씩 VK”는 한글 입력에 적합하지 않다.

### 해결

- Windows에서는 **`KEYEVENTF_UNICODE`** 로 **UTF-16 코드 유닛**을 주입한다.
- **VK 매핑으로 한글을 보내지 않는다.**
- surrogate pair(이모지 등)도 UTF-16 규칙에 맞게 순서대로 전송한다.

enigo / raw `SendInput` 래퍼를 쓰더라도, 한글·비ASCII 경로는 UNICODE 플래그 경로만 사용한다.

---

## 6. Clipboard ring self-trigger (클립보드 링 자기 트리거)

**적용 Phase**: Phase 1 — ClipboardPlain injector (+ 클립보드 감시/링)

### 문제

앱이 스니펫을 클립보드에 쓰는 순간, 자체 **클립보드 감시(ring/history)** 가 그 변경을 “사용자 복사”로 인식해:

- 히스토리에 스니펫이 중복 적재되거나
- 재귀적으로 injector가 다시 돌거나
- Win+V 클라우드/로컬 히스토리가 오염된다

### 해결

1. **`IS_INJECTING: AtomicBool`** 가드를 **`Ordering::SeqCst`** 로 둔다.
2. 주입 시작 시 `true`, 감시 콜백은 `true`이면 **기록/재트리거를 건너뛴다**.
3. 클립보드 **복원 완료 후 약 50ms** 뒤에 `false`로 클리어한다 (복원 직후 늦게 도착하는 WM_CLIPBOARDUPDATE 대비).
4. 가능하면 클립보드에 쓸 때 **`ExcludeClipboardContentFromMonitorProcessing`** 를 사용해, Windows 클립보드 기록(Win+V)에 **주입용 내용이 남지 않게** 한다.

권장 순서:

```text
IS_INJECTING = true (SeqCst)
→ backup / set (ExcludeClipboardContentFromMonitorProcessing 권장)
→ Ctrl+V
→ sleep 80–120ms
→ restore
→ sleep ~50ms
→ IS_INJECTING = false (SeqCst)
```

함정 1·2와 동일 injector 경로에서 한 트랜잭션으로 묶는다.

---

## Phase ↔ 함정 매핑 (빠른 참조)

| 함정 | 요약 | 강제 Phase |
|------|------|------------|
| 1 | Modifier residual → 30–50ms 및/또는 Alt/Shift Up | Phase 1 injector |
| 2 | Clipboard restore → 80–120ms 후 복원 (설정화 예정) | Phase 1 injector |
| 3 | UIPI → 기본 일반 권한 + 선택 elevated + UI 배지 | 설정 / UI |
| 4 | HID MI_01 + `[u8;65]` Report ID `0x03` + `03 fd fe ff` | Phase 2 HID |
| 5 | Hangul → `KEYEVENTF_UNICODE` only | Phase 4 SimulatedTyping |
| 6 | Ring self-trigger → `IS_INJECTING` SeqCst + 50ms clear + Exclude… | Phase 1 injector |

---

## 에이전트 준수 규칙

1. 해당 Phase 코드를 작성·리뷰할 때 **이 문서의 해당 절을 먼저 읽고** 반영한다.
2. 타이밍 상수(30–50 / 80–120 / 50ms)를 임의로 0으로 줄이거나 제거하지 않는다. 변경 시 이 가이드와 설정 스키마를 함께 갱신한다.
3. 제품 기능 추가와 무관하게, injector/HID/SimulatedTyping 경로의 **회귀 테스트·수동 확인**에 위 체크를 포함한다.
