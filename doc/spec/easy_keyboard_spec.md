# 이지키보드 (EasyKeyboard) 아키텍처 및 기술 명세서

**프로젝트 개요:** `514C:8850` (16키 3노브 3모드 매크로 패드) 하드웨어 제어와 **대용량·다국어 텍스트 스니펫 원클릭 자동 붙여넣기**를 결합한 오픈소스 데스크톱/웹 하이브리드 유틸리티입니다.  
**권장 저장소 이름:** `github.com/namoman/easy-keyboard`  
**권장 라이선스:** `MIT License`

---

## 1. 핵심 설계 철학: 왜 "하드웨어 + 호스트 브릿지" 구조인가?

`514C:8850` 매크로 패드의 온보드 메모리는 키당 최대 **18개의 HID 키코드 시퀀스**만 저장할 수 있으며, 한글(IME 조합)이나 긴 문장, 줄바꿈이 포함된 상용구를 하드웨어 패킷만으로 직접 타이핑하는 데 한계가 있습니다 ([ch57x-keyboard-tool Issue #153](https://github.com/kriomant/ch57x-keyboard-tool/issues/153)).

따라서 **이지키보드(EasyKeyboard)**는 두 계층을 결합한 하이브리드 구조로 동작합니다.

| 구분 | 동작 방식 | 저장 위치 | 적합한 용도 |
| :--- | :--- | :--- | :--- |
| **모드 A: 하드웨어 직접 매핑** | `03 fd` 65바이트 HID 프로토콜로 패드 플래시 메모리에 키/마우스/RGB 직접 기록 ([MINIKeyboardWeb](https://github.com/hobbyworker/MINIKeyboardWeb)) | 매크로 패드 온보드 메모리 | 단축키(`Ctrl+C`), 미디어 제어, 마우스 휠/이동, 단일 키 입력 |
| **모드 B: 텍스트 스니펫 브릿지** | 패드의 키를 트리거 코드(`F13~F24` 또는 희귀 조합키)로 매핑하고, 로컬 백그라운드 데몬이 이를 감지해 등록된 텍스트를 즉시 붙여넣기 | PC 로컬 암호화 DB (`snippets.enc.json`) | 긴 상용구, 이메일/보고서 템플릿, 코드 스니펫, 한글 문장 |

---

## 2. 기술 스택 및 언어 구성

배포 파일 크기가 작고(약 5~8MB) 메모리 점유율이 낮으며, 기존 오픈소스 자산을 그대로 활용할 수 있는 **Tauri v2 (Rust + Svelte 5 + TypeScript)** 구성을 권장합니다.

| 계층 | 기술 스택 | 선택 이유 및 역할 |
| :--- | :--- | :--- |
| **데스크톱 프레임워크** | **Tauri v2** | Electron 대비 가벼운 리소스 점유율, 멀티 윈도우(설정창 + 투명 OSD 오버레이) 및 시스템 트레이 기본 지원 |
| **프론트엔드 (UI)** | **TypeScript + Svelte 5 + Vite** | [MINIKeyboardWeb](https://github.com/hobbyworker/MINIKeyboardWeb) (MIT)의 `<minikeyboard-app>` 컴포넌트 및 레이아웃 렌더링 코드를 그대로 이식·확장 가능 |
| **백엔드 (Core/Daemon)** | **Rust (2024 Edition)** | • `hidapi`: USB-C 유선 연결 시 `514C:8850` 65바이트 패킷 읽기/쓰기 ([ch57x-keyboard-tool](https://github.com/kriomant/ch57x-keyboard-tool) 및 [Macrocli](https://github.com/USERNAME-STOCK/Macrocli) 구조 참고)<br>• `tauri-plugin-global-shortcut`: 패드 키 입력 실시간 후킹<br>• `arboard` + `enigo`: 클립보드 백업/복원 및 `Ctrl+V` 시뮬레이션 |
| **스타일링** | **Tailwind CSS** | 다크 모드 설정 UI 및 반투명 OSD(On-Screen Display) 레이아웃 구성 |

---

## 3. 주요 기능 명세

### 3.1 텍스트 스니펫 엔진 (핵심 기능)
1. **원클릭 텍스트 등록:** UI에서 16개 키(레이어 1~3, 총 48슬롯) 중 하나를 클릭하고 원하는 텍스트(여러 줄, 한글, 이모지, 마크다운 등)를 붙여넣어 저장합니다.
2. **자동 트리거 매핑:** 사용자가 텍스트 스니펫 모드로 키를 저장하면, 이지키보드가 USB를 통해 해당 물리 키에 자동으로 트리거 키코드(유선/2.4GHz 동글 모드용 `F13~F24` 또는 블루투스 호환용 `Ctrl+Alt+Shift+F1~F12`)를 기록합니다 ([ch57x-keyboard-tool Issue #153](https://github.com/kriomant/ch57x-keyboard-tool/issues/153)).
3. **비파괴 클립보드 주입 (Non-destructive Paste):**
   - 사용자가 매크로 패드 키를 누름 → 백엔드 데몬이 트리거 감지
   - 현재 OS 클립보드 내용을 메모리에 임시 백업 (`arboard`)
   - 지정된 텍스트 스니펫을 클립보드에 적재 후 즉시 `Ctrl+V` (`Shift+Insert`) 이벤트 전송 (`enigo`)
   - 50~100ms 후 원래 클립보드 내용으로 자동 복원 (기존에 복사해 둔 데이터 유지)
4. **동적 변수 치환:** `{{date}}` (오늘 날짜), `{{time}}`, `{{clipboard}}` (현재 클립보드 내용 포함하여 템플릿 완성) 태그를 실시간 치환합니다.

### 3.2 하드웨어 설정 및 프로토콜 제어 (`514C:8850`)
1. **실시간 설정 동기화 (Read-back):** 패드 연결 시 현재 저장된 3개 레이어(각 25개 바인딩 리포트)를 읽어와 화면에 표시하고, 저장 직후 다시 읽어와 정상 기록 여부를 검증합니다 ([MINIKeyboard 1.2.0](https://hobbyworker.me/en/projects/minikeyboard/1-2-0/)).
2. **확장 하드웨어 기능:**
   - **키별 정밀 딜레이:** 키 시퀀스 내 각 키 입력 사이마다 `0~6000ms` (16-bit Big-Endian) 딜레이 지정 ([Macrocli](https://github.com/USERNAME-STOCK/Macrocli)).
   - **마우스 확장 제어:** 클릭뿐만 아니라 노브 회전 시 스크롤 배속(`0x03`, `0xfd` 등) 및 X/Y축 커서 이동 바인딩 지원 ([ch57x-keyboard-tool Issue #153](https://github.com/kriomant/ch57x-keyboard-tool/issues/153)).
   - **키별 RGB 설정:** 유선 연결 시 사용할 키별 색상 및 조명 모드 제어.

### 3.3 실시간 OSD (On-Screen Display) 오버레이
- 화면 구석에 반투명(`Always-on-top`, 클릭 통과 `IgnoreCursorEvents` 토글 지원) 미니 패드 위젯을 띄워, 각 키에 어떤 텍스트 스니펫이나 단축키가 들어 있는지 사용자 지정 라벨(`labels.json`)로 보여줍니다 ([Macrocli](https://github.com/USERNAME-STOCK/Macrocli)).

---

## 4. 무선 모드(블루투스 vs 2.4GHz) 호환성 설계

`514C:8850` 펌웨어는 블루투스(BT) 모드에서 `F13~F24` 키코드를 전송하지 못하는 하드웨어 제약이 있습니다 ([ch57x-keyboard-tool Issue #153](https://github.com/kriomant/ch57x-keyboard-tool/issues/153)). 이를 해결하기 위해 **듀얼 트리거 프로파일**을 제공합니다.

| 연결 모드 | 트리거 키코드 할당 전략 | 비고 |
| :--- | :--- | :--- |
| **유선 / 2.4GHz 동글 모드** | `F13` ~ `F24` + 조합키 (`Ctrl+F13` 등) | 일반 작업 단축키와 절대 충돌하지 않는 가장 깔끔한 방식 |
| **블루투스(BT) 호환 모드** | `Ctrl + Alt + Shift + [영문/숫자]` | 블루투스 연결 시에도 100% 전송되는 조합키를 자동 할당하여 스니펫 실행 보장 |

---

## 5. 보안 설계 (Security Specification)

퍼블릭 레포지토리로 공개할 때 사용자와 개발자 모두를 보호하기 위한 보안 요구사항입니다.

### 5.1 로컬 텍스트 저장소 암호화 및 민감정보 보호
- **OS 키체인 연동 암호화 (At-Rest Encryption):** 사용자가 자주 쓰는 텍스트에 사내 시스템 계정명, 양식, 개인정보 등이 포함될 수 있으므로 평문 JSON 대신 Windows DPAPI / OS Credential Manager에서 파생한 키로 **AES-256-GCM 암호화**하여 로컬에 저장합니다.
- **민감 스니펫 마스킹 (`Secret Flag`):** 특정 슬롯을 '민감 텍스트'로 지정하면 UI와 OSD 오버레이에서 내용을 `••••••••`로 마스킹하고, 붙여넣기 직후 임시 클립보드 버퍼를 즉시 제로화(`zeroize` 크레이트 활용)한 뒤 이전 클립보드로 덮어씁니다.
- **Git 커밋 사고 방지:** `.gitignore`에 사용자 프로필(`*.profile.json`, `labels.json`, `snippets.enc`)을 기본 등록하여 개인 설정이나 업무용 상용구가 퍼블릭 레포에 커밋되지 않도록 차단합니다.

### 5.2 Tauri 및 USB HID 실행 보안
- **네트워크 완전 격리 (Zero-Network CSP):** `tauri.conf.json`의 Content Security Policy(CSP)에서 외부 통신(`connect-src 'none'`)을 원천 차단하여, 입력한 텍스트나 키 로그가 외부로 전송될 수 없음을 코드 레벨에서 증명합니다 ([MINIKeyboard](https://apps.microsoft.com/detail/9n64pj6qht0p)).
- **최소 권한(Capabilities) 분리:** Tauri v2의 `capabilities` 설정을 통해 메인 설정 창에만 HID 제어 및 파일 읽기/쓰기 권한을 부여하고, 투명 OSD 창에는 읽기 전용 이벤트 수신 권한만 부여합니다.
- **HID 패킷 검증 (Bounds Checking):** 기기로 전송하는 모든 패킷은 정확히 `65바이트`(`0x03 0xfd` 헤더 시작, 최대 18키 시퀀스) 규격과 오프셋 범위를 Rust 타입 시스템으로 검증하여 잘못된 패킷으로 인한 기기 벽돌(Bricking) 현상을 방지합니다 ([ch57x-keyboard-tool Issue #153](https://github.com/kriomant/ch57x-keyboard-tool/issues/153)).

### 5.3 오픈소스 라이선스 및 공급망 보안
- **라이선스 준수 (Clean-room Implementation):**
  - [MINIKeyboardWeb](https://github.com/hobbyworker/MINIKeyboardWeb)과 [ch57x-keyboard-tool](https://github.com/kriomant/ch57x-keyboard-tool)은 **MIT 라이선스**이므로 `THIRD_PARTY_NOTICES.md`에 저작권 고지를 포함하고 코드를 활용합니다.
  - [Macrocli](https://github.com/USERNAME-STOCK/Macrocli)는 저장소에 명시적 오픈소스 라이선스 파일이 없으므로, 소스 코드를 직접 복사하지 않고 공개된 USB HID 프로토콜 구조와 기능 아이디어만 클린룸 방식으로 직접 구현합니다.
- **투명한 빌드 파이프라인 (GitHub Actions + Attestation):** 중국산 매크로 툴의 가장 큰 불신 요인인 바이러스 오탐을 해소하기 위해, GitHub Actions 워크플로에서 릴리스 바이너리를 자동 빌드하고 `actions/attest-build-provenance`로 빌드 출처 증명 및 SHA-256 체크섬을 함께 배포합니다.

---

## 6. 퍼블릭 레포지토리 디렉터리 구조

```text
easy-keyboard/
├── .github/
│   └── workflows/
│       ├── ci.yml                  # Rust clippy, cargo test, Svelte check
│       └── release.yml             # Windows/Linux/macOS 자동 빌드 및 릴리스 배포
├── src/                            # 프론트엔드 (Svelte 5 + TypeScript)
│   ├── lib/
│   │   ├── components/
│   │   │   ├── KeypadCanvas.svelte # 16키 3노브 시각적 레이아웃 (MINIKeyboardWeb 기반)
│   │   │   ├── SnippetEditor.svelte# 텍스트 붙여넣기·변수 태그·트리거 모드 편집기
│   │   │   ├── HardwareTab.svelte  # 단축키, 노브, 마우스 이동, 키별 RGB 설정
│   │   │   └── OsdOverlay.svelte   # 반투명 항상 위 OSD 치트시트 창
│   │   ├── stores/                 # 키 매핑 및 스니펫 상태 관리
│   │   └── ipc.ts                  # Tauri Rust 백엔드 호출 래퍼
│   ├── App.svelte
│   └── main.ts
├── src-tauri/                      # 백엔드 (Rust)
│   ├── capabilities/
│   │   ├── main-window.json        # 설정창 권한 (HID, 저장소, 단축키 등록)
│   │   └── osd-window.json         # OSD창 최소 권한
│   ├── src/
│   │   ├── hid/
│   │   │   ├── mod.rs
│   │   │   └── k8850.rs            # 514C:8850 65바이트 프로토콜 읽기/쓰기/검증
│   │   ├── snippet/
│   │   │   ├── injector.rs         # 글로벌 단축키 감지 -> 클립보드 백업 -> 붙여넣기 -> 복원
│   │   │   └── crypto.rs           # AES-256-GCM + OS 키체인 로컬 암호화
│   │   └── main.rs                 # 시스템 트레이, 멀티 윈도우 초기화
│   ├── Cargo.toml
│   └── tauri.conf.json             # CSP(네트워크 차단) 및 창 설정
├── THIRD_PARTY_NOTICES.md          # MIT 오픈소스 출처 표기
├── .gitignore                      # 개인 스니펫 및 라벨 데이터 제외
├── LICENSE                         # MIT License
└── README.md                       # 프로젝트 소개 및 사용법
```

---

## 7. 초기 세팅 명령어 및 구현 로드맵

### Step 1. 프로젝트 생성 및 필수 패키지 설치
```bash
# 1. Tauri v2 + Svelte + TypeScript 프로젝트 생성
npm create tauri-app@latest easy-keyboard -- --template svelte-ts
cd easy-keyboard

# 2. Tauri 글로벌 단축키 플러그인 추가
npm run tauri add global-shortcut

# 3. Rust 백엔드 크레이트 추가 (src-tauri 폴더 이동)
cd src-tauri
cargo add hidapi arboard enigo aes-gcm keyring zeroize serde_json
```

### Step 2. 단계별 개발 마일스톤 (4단계)
1. **Milestone 1 (텍스트 스니펫 코어):** Tauri 시스템 트레이 앱 구동 + `tauri-plugin-global-shortcut`으로 `F13~F24` 입력 시 `arboard`와 `enigo`를 통해 클립보드 백업 후 지정된 텍스트를 즉시 붙여넣고 복원하는 엔진 구현.
2. **Milestone 2 (`514C:8850` HID 브릿지):** `k8850.rs` 모듈 작성 (65바이트 `03 fd` 리포트 송수신) 및 UI에서 스니펫 저장 시 패드의 해당 키를 트리거 키코드로 자동 매핑하는 연동 구현.
3. **Milestone 3 (시각적 UI 및 RGB/마우스 제어):** [MINIKeyboardWeb](https://github.com/hobbyworker/MINIKeyboardWeb)의 키패드 UI 컴포넌트를 이식하고, 키별 딜레이·마우스 이동·키별 RGB 설정 탭 통합.
4. **Milestone 4 (OSD 오버레이 및 보안 패키징):** 두 번째 투명 윈도우(`OsdOverlay.svelte`) 추가, 로컬 스니펫 암호화(`crypto.rs`) 적용, GitHub Actions 자동 빌드 파이프라인 구성.
