# EasyKeyboard — 로컬 암호화 저장

## 개요
스니펫은 평문 JSON이 아니라 **AES-256-GCM**으로 암호화된 `snippets.enc`에 저장된다.  
키는 **OS 자격 증명 저장소**(Windows Credential Manager / DPAPI 경로의 `keyring`)에만 둔다.

## 파일 위치
| 항목 | 경로 |
|------|------|
| 암호문 | `%APPDATA%\com.namoman.easy-keyboard\snippets.enc` (Tauri `app_data_dir`) |
| AES 키 | keyring service `com.namoman.easy-keyboard` / account `snippets-aes-key-v1` |

`.gitignore`에 `snippets.enc*` 가 포함되어 있다.

## 포맷
```
nonce (12 bytes) || ciphertext + GCM tag
```
평문은 `SnippetStore`의 `serde_json` 바이트.

## Secret 슬롯
- UI/OSD: 본문 블러·라벨만 표시
- 주입 직후 임시 버퍼 `zeroize` (pitfall 6과 함께 `IS_INJECTING` 유지)
- 디스크에는 암호화 상태로만 존재

## 관련 코드
- `src-tauri/src/snippet/crypto.rs`
- `src-tauri/src/snippet/persist.rs`
