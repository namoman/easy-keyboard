# Third-Party Notices

EasyKeyboard는 아래 오픈소스 구성 요소와 참고 프로젝트에 의존하거나 영감을 받았습니다.
본 프로젝트 코드는 MIT로 배포되며, 각 구성 요소의 라이선스를 함께 준수합니다.

## Runtime / Framework

| 구성 요소 | 라이선스 | 비고 |
| :--- | :--- | :--- |
| [Tauri](https://github.com/tauri-apps/tauri) | MIT / Apache-2.0 | 데스크톱 셸 |
| [Svelte](https://github.com/sveltejs/svelte) / [SvelteKit](https://github.com/sveltejs/kit) | MIT | UI |
| [Vite](https://github.com/vitejs/vite) | MIT | 프론트엔드 빌드 |
| [Tailwind CSS](https://github.com/tailwindlabs/tailwindcss) | MIT | 디자인 토큰/유틸리티 |
| [serde](https://github.com/serde-rs/serde) / serde_json | MIT / Apache-2.0 | 직렬화 |
| [hidapi](https://github.com/ruabmbua/hidapi-rs) | MIT | USB HID |
| [arboard](https://github.com/1Password/arboard) | MIT / Apache-2.0 | 클립보드 |
| [enigo](https://github.com/enigo-rs/enigo) | MIT | 입력 시뮬레이션 |
| [aes-gcm](https://github.com/RustCrypto/AEADs) | MIT / Apache-2.0 | 스니펫 암호화 |
| [keyring](https://github.com/open-source-cooperative/keyring-rs) | MIT / Apache-2.0 | OS 키체인 |
| [zeroize](https://github.com/RustCrypto/utils) | MIT / Apache-2.0 | 민감 메모리 소거 |
| [tauri-plugin-global-shortcut](https://github.com/tauri-apps/plugins-workspace) | MIT / Apache-2.0 | 전역 단축키 |

## Reference projects (프로토콜/UX 참고 — 코드 미복사)

| 프로젝트 | 라이선스 | 용도 |
| :--- | :--- | :--- |
| [hobbyworker/MINIKeyboardWeb](https://github.com/hobbyworker/MINIKeyboardWeb) | MIT | 웹 UI/레이아웃 참고 |
| [hobbyworker/MINIKeyboardWin](https://github.com/hobbyworker/MINIKeyboardWin) | MIT | 읽기/쓰기·검증 흐름 참고 |
| [kriomant/ch57x-keyboard-tool](https://github.com/kriomant/ch57x-keyboard-tool) | MIT / Apache-2.0 | `514C:8850` 65바이트 프로토콜 참고 |
| [USERNAME-STOCK/Macrocli](https://github.com/USERNAME-STOCK/Macrocli) | 명시 없음 | 딜레이/OSD 컨셉 참고 — **클린룸 재구현, 코드 복사 금지** |

로컬 참고 클론은 `ref/`에 보관하며 gitignore 대상입니다.
