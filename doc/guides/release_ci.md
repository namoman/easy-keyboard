# EasyKeyboard — CI / Release

## 워크플로
| 파일 | 트리거 | 내용 |
|------|--------|------|
| `.github/workflows/ci.yml` | push/PR | `npm run check`+build (Ubuntu), `cargo check`+`test --lib` (Windows) |
| `.github/workflows/release.yml` | tag `v*` | Tauri Windows 번들, SHA-256 체크섬 artifact, build provenance attestation |

## 로컬 검증
```bash
npm run check
cd src-tauri && cargo check && cargo test --lib
```

## 릴리스 절차 (골격)
1. `git tag v0.x.y && git push origin v0.x.y`
2. Actions → Release 초안 생성
3. `checksums-sha256` artifact와 attestation으로 무결성 확인
4. 초안 검토 후 Publish

## 주의
- 1차 타깃은 Windows x64. Linux/macOS 러너는 CI 프론트만.
- 코드 사이닝 인증서는 저장소 secrets로 추후 연결 (`TAURI_SIGNING_*`).
