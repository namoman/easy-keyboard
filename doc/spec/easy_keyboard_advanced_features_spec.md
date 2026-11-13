# 이지키보드 (EasyKeyboard) 확장 기능 상세 명세서 (`ADVANCED_FEATURES_SPEC.md`)

> 본 문서는 기본 아키텍처 명세(`easy_keyboard_spec.md`)와 UI/UX 명세(`easy_keyboard_ui_ux_spec.md`)에 이어, 실사용 생산성을 극대화하는 **5대 핵심 확장 기능**의 백엔드(Rust) 동작 원리, 데이터 구조, 프론트엔드(Svelte 5) 연동 규격을 정의합니다.

---

## 1. 활성 프로그램별 자동 프로필 전환 (App-Aware Dynamic Profile)

매크로 패드의 물리 레이어 버튼을 직접 누르지 않아도, 현재 포커스된 프로그램(`Hwp.exe`, `EXCEL.EXE`, `Code.exe`, 브라우저 등)에 맞춰 동일한 물리 키(`F13~F24` 트리거)가 자동으로 해당 프로그램 전용 스니펫 세트로 전환됩니다.

### 1.1 동작 원리 (Rust 백엔드)
- **하드웨어 재기록 없음 (Zero Flash Wear):** 프로그램이 바뀔 때마다 USB HID로 패드의 플래시 메모리를 다시 쓰지 않습니다. 패드는 항상 고정된 트리거(`F13~F24` 또는 블루투스 조합키)만 내보내고, 호스트의 Rust 데몬이 **현재 활성 창의 실행 파일명(`Process Name`)**을 확인해 매칭된 프로필의 스니펫을 실행합니다.
- **포커스 감지 방식:**
  - **Windows:** `windows-sys` 크레이트의 `SetWinEventHook` (`EVENT_SYSTEM_FOREGROUND`) 또는 경량 폴링(150ms 주기)으로 `GetForegroundWindow()` → `GetWindowThreadProcessId()` → 실행 파일명(예: `hwp.exe`) 및 창 제목(`Window Title`)을 추출합니다.
  - **이벤트 브로드캐스트:** 활성 프로그램이 바뀌어 프로필이 전환되면 Tauri 이벤트(`app://profile-switched`)를 발생시켜 `OsdOverlay.svelte`의 라벨 그리드를 즉시 갱신합니다.

### 1.2 데이터 스키마 (`ProfileRule`)
```json
{
  "profile_id": "work-hwp",
  "name": "한글 공문서 작성",
  "priority": 10,
  "match_rules": {
    "executables": ["hwp.exe", "hword.exe"],
    "title_contains": []
  },
  "manual_lock": false
}
```
- **우선순위 폴백 (Fallback):** 현재 활성 창과 일치하는 규칙이 없으면 항상 **`Default (기본 글로벌 프로필)`**로 동작하며, 상단 헤더나 OSD에서 `[📌 현재 프로필 고정]`을 켜면 자동 전환을 일시 중지합니다.

---

## 2. 노브(다이얼) 전용 '클립보드 히스토리 휠' (Clipboard Ring Wheel)

3개의 회전 노브 중 하나(기본 추천: `Knob 2` 또는 `Knob 3`)를 **클립보드 타임머신**으로 할당하여, 이전에 복사했던 텍스트들을 다이얼로 돌려가며 골라 붙여넣는 기능입니다.

### 2.1 하드웨어 트리거 할당 (`514C:8850`)
- 노브 슬롯을 '클립보드 휠 모드'로 설정하면, 이지키보드가 해당 노브의 3가지 동작을 전용 단축키로 자동 매핑합니다 ([ch57x-keyboard-tool Issue #153](https://github.com/kriomant/ch57x-keyboard-tool/issues/153)):
  - **좌회전 (`CCW`):** 이전 복사 항목 선택 (예: `Ctrl + Alt + Shift + PageUp`)
  - **우회전 (`CW`):** 다음 복사 항목 선택 (예: `Ctrl + Alt + Shift + PageDown`)
  - **노브 누름 (`Press`):** 현재 선택된 히스토리 항목 즉시 붙여넣기 (예: `Ctrl + Alt + Shift + Enter`)

### 2.2 백엔드 링 버퍼 및 보안 규칙 (`src-tauri/src/snippet/clipboard_ring.rs`)
- **메모리 전용 링 버퍼 (`VecDeque`):** 최근 복사한 텍스트를 최대 **20개**까지 메모리(`RAM`)에만 보관하며, 앱 종료 시 디스크에 남기지 않고 소거합니다(민감 데이터 유출 방지).
- **자가 주입 무시 (Self-Injection Guard):** 이지키보드 자신이 스니펫을 붙여넣기 위해 클립보드를 일시 변경할 때는 내부 플래그(`is_injecting: AtomicBool`)를 세워 히스토리 링에 중복 수집되지 않도록 차단합니다.
- **OSD 휠 오버레이 연동:** 노브를 한 칸이라도 돌리면 `OsdOverlay.svelte`가 3초간 **클립보드 캐러셀 뷰**로 전환되어, 선택 중인 항목의 앞뒤 문맥(최대 3줄 미리보기)을 보여주고 노브를 누르는 순간 커서 위치에 주입합니다.

---

## 3. 붙여넣기 주입 방식 분기 (Injection Mode Routing)

대상 프로그램의 특성(서식 유지 여부, 보안 프로그램의 클립보드 차단 여부)에 따라 슬롯별로 3가지 텍스트 주입 엔진 중 하나를 선택할 수 있습니다.

| 주입 모드 | 내부 코드명 | 동작 메커니즘 | 권장 사용처 |
| :--- | :--- | :--- | :--- |
| **1. 순수 텍스트 붙여넣기 (기본)** | `ClipboardPlain` | 클립보드에 UTF-8 평문만 적재 후 `Ctrl+V` 전송 → 원래 클립보드 복원 | 한글(`Hwp`), 엑셀, 그룹웨어 기안기 (원본 폰트/표 깨짐 방지) |
| **2. 터미널 붙여넣기** | `ClipboardTerminal`| 클립보드 적재 후 `Ctrl+Shift+V` 또는 `Shift+Insert` 전송 | Windows Terminal, VS Code 내장 터미널, WSL, PuTTY |
| **3. 직접 타이핑 시뮬레이션** | `SimulatedTyping` | 클립보드를 건드리지 않고 `enigo.text()` (Windows `SendInput`의 `KEYEVENTF_UNICODE`)로 글자 단위 직접 타건 | 클립보드 붙여넣기(`Ctrl+V`)가 막힌 보안 웹페이지, VM 콘솔, 비밀번호 입력창 |

### 3.1 타이핑 시뮬레이션(`SimulatedTyping`) 세부 옵션
- **타건 간격 (`Typing Delay`):** 기본 `5ms~15ms` 설정 가능. 너무 빠른 입력을 매크로로 차단하거나 글자가 누락되는 레거시 프로그램에서는 `20ms`로 높여 안정적으로 입력합니다.

---

## 4. 포스트 액션 매크로 파서 (`{{cursor}}`, `{{tab}}`, `{{enter}}`)

단순히 텍스트 덩어리를 붙여넣는 것을 넘어, **붙여넣기 직후 커서를 원하는 위치로 옮기거나 다음 입력칸(셀)으로 넘어가며 양식을 자동으로 채우는** 템플릿 파서입니다.

### 4.1 지원 제어 태그 규격

| 태그 | 동작 설명 | 활용 예시 |
| :--- | :--- | :--- |
| **`{{cursor}}`** | 전체 텍스트를 붙여넣은 뒤, `{{cursor}}`가 있던 자리로 **`Left Arrow` 키 이벤트를 역산하여 전송**해 커서를 정확히 그 위치에 놓습니다. | `[월미관리-{{cursor}}] 준공검사원 제출` → 붙여넣기 후 대괄호 안에 커서가 위치해 문서번호만 바로 타이핑 |
| **`{{tab}}`** | 태그를 기준으로 텍스트를 분할하여 앞부분 붙여넣기 → `Tab` 키 전송(50ms 대기) → 뒷부분 붙여넣기를 순차 실행합니다. | `이남호{{tab}}월미관리팀{{tab}}과장` → 엑셀이나 웹 표의 3개 칸을 한 번에 채움 |
| **`{{enter}}`** | 줄바꿈 문자(`\n`) 대신 실제 키보드 `Enter` 키 이벤트를 발생시킵니다. | 메신저에서 두 문장을 각각 별도 말풍선으로 연속 전송할 때 활용 |
| **`{{wait:500}}`**| 다단계 입력 도중 지정한 밀리초(`ms`)만큼 대기합니다. | 페이지 로딩이나 팝업 전환 후 다음 텍스트를 넣어야 할 때 활용 |

### 4.2 `{{cursor}}` 역산 알고리즘 (`src-tauri/src/snippet/parser.rs`)
1. 원본 문자열에서 `{{cursor}}` 토큰을 제거한 최종 문자열 `S`를 생성합니다.
2. `{{cursor}}` 뒤쪽에 남은 문자열의 **유니코드 스칼라(Grapheme Cluster) 글자 수 $N$**을 계산합니다 (이모지나 개행 문자 `\r\n`은 에디터 커서 이동 기준 1칸으로 정규화).
3. `S`를 클립보드로 붙여넣은 직후, `enigo`를 통해 `Key::LeftArrow`를 $N$회 연속 전송하여 커서를 정확한 빈칸 위치에 안착시킵니다.

---

## 5. 암호화 프로필 백업(`.ekb`) 및 실물 키캡 라벨 시트 생성기

### 5.1 휴대용 암호화 번들 (`.ekb` Export / Import)
집 PC와 회사 PC 간에 설정을 안전하게 옮기거나 GitHub 이슈에 템플릿을 공유할 수 있는 단일 파일 포맷입니다.
- **내보내기 모드 2종:**
  1. **개인 백업용 (암호화 `.ekb`):** 사용자가 입력한 마스터 비밀번호로 `Argon2id` 키 파생 후 `AES-256-GCM`으로 전체 스니펫·라벨·하드웨어 키맵을 암호화하여 저장합니다.
  2. **퍼블릭 공유용 (클린 `.json`):** `Secret(민감 텍스트)` 플래그가 켜진 슬롯은 자동으로 비우고, 일반 단축키와 공개 상용구만 담아 다른 사용자에게 배포할 수 있게 내보냅니다.

### 5.2 실물 키캡 인쇄용 라벨 시트 생성기 (`Keycap Legend Print View`)
알리익스프레스 매크로 패드 특유의 투명 뚜껑 키캡(Relegendable Keycaps)이나 키캡 위 스티커에 끼워 넣을 수 있도록, 현재 설정된 16개 키 라벨을 **실제 치수(1:1 스케일) 인쇄용 도안**으로 출력합니다.
- **규격 프리셋:**
  - `12 × 12 mm` (표준 MX 키캡 상단 면적)
  - `13.5 × 13.5 mm` (투명 이중 키캡 내부 종이 삽입 규격)
- **출력 방식:** UI 상단 메뉴에서 `[🖨️ 키캡 라벨지 인쇄 / 이미지 저장]` 클릭 시, 레이어 1~3의 4×4 그리드가 절취선(`dashed border`), 슬롯 번호, 컬러 도트와 함께 A4 정밀 배율(`@media print` CSS `mm` 단위 고정) 또는 고해상도 PNG로 즉시 렌더링됩니다.

---

## 6. 기존 프로젝트 구조에 추가되는 모듈 매핑

기존 `easy_keyboard_spec.md`의 디렉터리 구조에 아래 파일들만 추가하면 5대 기능이 깔끔하게 통합됩니다.

```text
easy-keyboard/
├── src/lib/components/
│   ├── ProfileBar.svelte           # [추가] 프로그램별 자동 프로필 규칙 설정 및 고정(Pin) 토글
│   ├── ClipboardWheelModal.svelte  # [추가] OSD 내부 노브 회전 시 뜨는 클립보드 히스토리 휠 뷰
│   └── KeycapPrintSheet.svelte     # [추가] 12x12mm / 13.5x13.5mm 실물 키캡 인쇄용 도안 생성기
└── src-tauri/src/
    ├── context/
    │   └── window_watcher.rs       # [추가] 활성 창(hwp.exe, excel.exe 등) 감지 및 프로필 자동 스위칭
    └── snippet/
        ├── parser.rs               # [추가] {{cursor}}, {{tab}}, {{wait}} 토큰 파싱 및 커서 역산기
        ├── clipboard_ring.rs       # [추가] 메모리 전용 최근 클립보드 20개 링 버퍼
        └── bundle.rs               # [추가] Argon2id + AES-256-GCM 기반 .ekb 파일 내보내기/가져오기
```
