# ⌨️ EasyKeyboard (이지키보드)

[License: MIT](./LICENSE)
[Tauri](https://v2.tauri.app/)
[Rust](https://www.rust-lang.org/)
[Svelte](https://svelte.dev/)
[지원 하드웨어](#지원-하드웨어)

**EasyKeyboard(이지키보드)**는 알리익스프레스 등에서 유통되는 16키 3노브 3모드 기계식 매크로 패드(`VID_514C&PID_8850`)를 위한 **오픈소스 하드웨어 설정기이자 원클릭 텍스트 스니펫(상용구) 자동 입력 도구**입니다.

제조사가 제공하는 출처 불명의 중국산 설정 프로그램 없이도 패드의 모든 하드웨어 기능(키 매핑, 노브, 마우스 이동/배속 스크롤, 키별 RGB)을 안전하게 제어할 수 있으며, 하드웨어의 18타 키코드 한계를 뛰어넘어 **한글 장문 상용구, 보고서/공문서 양식, 코드 스니펫을 키 하나로 즉시 붙여넣는 호스트 브릿지 엔진**을 제공합니다.

---

## ✨ 왜 이지키보드인가요?

1. **백신 오탐 및 외부 통신 제로 (Zero-Network & Auditable):**
   - `tauri.conf.json`의 CSP 설정(`connect-src 'none'`)으로 외부 네트워크 연결을 원천 차단했으며, GitHub Actions를 통해 투명하게 빌드됩니다.
2. **한글 장문 & 다중 줄 상용구 원클릭 입력 (Non-destructive Paste):**
   - 온보드 메모리(최대 18키)에 들어가지 않는 긴 한글 문장도 버튼 하나에 등록해 두면, 기존 클립보드 내용을 보존(백업 후 자동 복원)하면서 0.1초 만에 입력합니다.
3. **무선 3모드(유선 / 2.4GHz 동글 / 블루투스) 완벽 대응:**
   - 블루투스 모드에서 `F13~F24` 확장키가 전송되지 않는 `514C:8850` 펌웨어 제약을 우회하기 위해 **동글/유선 전용 프로필(`F13~F24`)**과 **블루투스 호환 프로필(충돌 없는 조합키)**을 원클릭으로 전환합니다 ([ch57x-keyboard-tool Issue #153](https://github.com/kriomant/ch57x-keyboard-tool/issues/153)).
4. **반투명 항상 위 OSD 치트시트 & 자동 프로필:**
   - 작업 화면 구석에 반투명 OSD 위젯을 띄워 각 키에 어떤 상용구가 들어 있는지 실시간으로 확인하고, 현재 활성화된 프로그램(한글, 엑셀, VS Code 등)에 맞춰 스니펫 세트를 자동 전환합니다.

---

## 🚀 주요 기능

### 📋 텍스트 스니펫 브릿지 (Host Daemon)
- **2-Step 등록:** 가상 키패드에서 원하는 키를 클릭하고 `Ctrl+V`로 텍스트만 붙여넣으면 트리거 키코드 할당부터 라벨 생성까지 자동 완료됩니다.
- **스마트 템플릿 태그:** `{{date}}` (오늘 날짜), `{{time}}`, `{{clipboard}}` 치환은 물론, 붙여넣기 후 원하는 괄호 안으로 커서를 되돌리는 `{{cursor}}`와 다음 칸으로 넘어가며 표를 채우는 `{{tab}}` 태그를 지원합니다.
- **주입 방식 3종 분기:** 서식 깨짐을 막는 `순수 텍스트 붙여넣기`, 터미널용 `Shift+Insert`, 붙여넣기(`Ctrl+V`)가 차단된 보안 입력창을 통과하는 `직접 타이핑 시뮬레이션`을 슬롯별로 선택할 수 있습니다.
- **노브 클립보드 히스토리 휠:** 다이얼을 돌려 방금 전 복사했던 최근 20개 텍스트 중 하나를 골라 바로 붙여넣습니다.
- **로컬 암호화 (`AES-256-GCM`):** OS 키체인(Windows DPAPI 등)과 연동해 스니펫 DB를 암호화하고, 민감 슬롯(`Secret`)은 화면 마스킹 및 클립보드 즉시 제로화(`zeroize`)를 수행합니다.

### ⚡ 하드웨어 직접 매핑 (`514C:8850` USB HID)
- **실시간 Read-back 검증:** 패드에 저장된 3개 레이어 설정을 즉시 읽어오고, 저장 후 바이트 단위로 재검증해 누락을 방지합니다 ([MINIKeyboard 1.2.0](https://hobbyworker.me/en/projects/minikeyboard/1-2-0/)).
- **키별 정밀 딜레이:** 최대 18타 시퀀스의 각 키 사이에 `0~6000ms` (16-bit Big-Endian) 지연 시간을 개별 설정합니다 ([Macrocli](https://github.com/USERNAME-STOCK/Macrocli)).
- **마우스 확장 제어:** 제조사 기본 앱에도 숨겨져 있는 **마우스 X/Y 커서 이동** 및 **휠 스크롤 배속(`1x~5x`)** 바인딩을 지원합니다 ([ch57x-keyboard-tool Issue #153](https://github.com/kriomant/ch57x-keyboard-tool/issues/153)).
- **키별 RGB 조명:** 유선 연결 시 작동하는 키별 색상 및 5가지 조명 효과를 설정합니다.

---

## 🛠️ 지원 하드웨어

| 모델 구분 | USB VID:PID | 물리 구성 | 연결 방식 | 지원 상태 |
| :--- | :--- | :--- | :--- | :--- |
| **K8850 (16-Key 3-Knob)** | `514C:8850` | 4×4 기계식 키 + 3 노브 | USB-C / 2.4G 동글 / BT 5.0 | ✅ 완벽 지원 (65-byte `03 fd` 프로토콜) |

> 💡 **참고:** 하드웨어 플래시 메모리 읽기/쓰기(초기 트리거 세팅 및 RGB 변경)는 **USB-C 유선 연결 상태**에서만 가능하며, 한 번 트리거를 저장해 두면 2.4GHz 동글이나 블루투스 무선 모드에서도 모든 텍스트 스니펫 기능이 동일하게 작동합니다.

---

## 📦 설치 및 빌드 방법

### 1. 릴리스 설치 파일 다운로드 (일반 사용자)
[GitHub Releases](https://github.com/namoman/easy-keyboard/releases) 페이지에서 최신 버전의 `EasyKeyboard_x64-setup.exe` 또는 포터블 실행 파일을 다운로드해 바로 실행할 수 있습니다. (별도 드라이버나 관리자 권한 불필요)

### 2. 소스 코드에서 직접 빌드하기 (개발자)

**사전 요구사항:** [Node.js 20+](https://nodejs.org/), [Rust (Cargo)](https://rustup.rs/)

```bash
# 저장소 클론
git clone https://github.com/namoman/easy-keyboard.git
cd easy-keyboard

# 프론트엔드 패키지 설치
npm install

# 개발 모드 실행 (Hot-reload 지원)
npm run tauri dev

# 배포용 프로덕션 빌드 (src-tauri/target/release/bundle 생성)
npm run tauri build
```

---

## 🗺️ 개발 로드맵 (Priority Roadmap)

| 단계 | 우선순위 | 핵심 구현 내용 |
| :--- | :--- | :--- |
| **Phase 1 (MVP)** | **P0 (필수 코어)** | • Tauri v2 + Svelte 5 기본 골격 및 시스템 트레이 상주<br>• `514C:8850` USB HID 읽기/쓰기(`k8850.rs`) 및 자동 트리거 매핑<br>• 비파괴 클립보드 백업·붙여넣기·복원 엔진 (`injector.rs`) |
| **Phase 2 (핵심 UX)** | **P1 (실사용 완성)** | • 반투명 항상 위 OSD 오버레이 (`OsdOverlay.svelte`) 및 고스트 모드<br>• `{{date}}`, `{{clipboard}}`, `{{cursor}}`, `{{tab}}` 템플릿 파서<br>• 붙여넣기 방식 3종 분기 (`순수 텍스트` / `터미널` / `직접 타이핑`) 및 로컬 암호화 |
| **Phase 3 (워크플로 확장)** | **P2 (고급 자동화)** | • 활성 프로그램(`hwp.exe`, `excel.exe`, `Code.exe`) 감지 자동 프로필 전환<br>• 노브 전용 클립보드 히스토리 휠 (최근 20개 링 버퍼)<br>• 암호화 프로필 번들(`.ekb`) 내보내기/가져오기 |
| **Phase 4 (부가 도구)** | **P3 (폴리싱)** | • 실물 키캡 삽입용 1:1(`12×12mm`) 라벨 시트 인쇄 뷰<br>• 다국어(한/영) 리소스 분리 및 커스텀 테마 |

---

## 📄 관련 설계 문서

- [easy_keyboard_spec.md](./doc/spec/easy_keyboard_spec.md) — 시스템 아키텍처, `514C:8850` 프로토콜, 보안 설계 명세
- [easy_keyboard_ui_ux_spec.md](./doc/spec/easy_keyboard_ui_ux_spec.md) — 화면 레이아웃, 디자인 토큰, 컴포넌트 인터랙션 명세
- [easy_keyboard_advanced_features_spec.md](./doc/spec/easy_keyboard_advanced_features_spec.md) — 5대 확장 기능 상세 구현 명세

---

## 🙏 감사의 글 및 오픈소스 고지 (Credits & Acknowledgments)

이지키보드는 아래 오픈소스 프로젝트들의 프로토콜 분석과 아키텍처에서 영감을 받아 제작되었습니다.
- **[hobbyworker/MINIKeyboardWeb](https://github.com/hobbyworker/MINIKeyboardWeb) & [MINIKeyboardWin](https://github.com/hobbyworker/MINIKeyboardWin)** (MIT License) — `514C:8850` 펌웨어 타입 11 읽기/쓰기 검증 및 웹 컴포넌트 레이아웃 설계 참고
- **[kriomant/ch57x-keyboard-tool](https://github.com/kriomant/ch57x-keyboard-tool)** (MIT/Apache-2.0 License) — [Issue #153](https://github.com/kriomant/ch57x-keyboard-tool/issues/153) (`@yawor`)의 `514C:8850` 65바이트 `03 fd` 패킷 구조 및 마우스 이동/배속 스크롤 리버스 엔지니어링 참고
- **[USERNAME-STOCK/Macrocli](https://github.com/USERNAME-STOCK/Macrocli)** — 키별 빅엔디안 딜레이 구조 및 OSD 오버레이 컨셉 참고

---

## 📜 라이선스 (License)

이 프로젝트는 **MIT License**에 따라 배포됩니다. 자세한 내용은 [LICENSE](./LICENSE) 파일을 참고하세요.
