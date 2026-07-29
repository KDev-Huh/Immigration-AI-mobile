# Task 0012 — GitHub 레포 + Actions CI/빌드 (iOS/Android)

- **상태**: in-progress (레포 생성만 남음 — `gh auth login` 필요)
- **의존**: 0011

## 완료

- `.github/workflows/ci.yml`
  - 프론트: typecheck + vitest + vite build
  - Rust: fmt / clippy `-D warnings` / test (ubuntu + webkit2gtk 등 시스템 의존성)
  - 모바일 타깃 컴파일 체크: `aarch64-linux-android`, `aarch64-apple-ios`
    → cfg 분기된 보안저장 구현(JNI / Keychain)이 실제로 빌드되는지 CI 에서 보장
- `.github/workflows/mobile-build.yml`
  - Android: `tauri android init` → APK(+태그 시 AAB). 서명 시크릿 있으면 업로드 키 구성
  - iOS: `tauri ios init` → 시뮬레이터 앱(서명 불필요) 아티팩트.
    `APPLE_*` 시크릿 있으면 실기기 IPA
  - `gen/` 은 커밋하지 않으므로 CI 가 매번 재생성
- 시크릿 게이팅은 **잡 레벨 env** 로 올림 — 스텝 자신의 env 는 그 스텝 `if:` 에서 못 읽는다.

## 로컬 검증 결과

| 항목 | 결과 |
|---|---|
| Android APK 빌드 (aarch64) | ✅ |
| Android 에뮬레이터 실행 | ✅ 3탭 렌더 |
| Android Keystore 저장 | ✅ prefs 에 암호문만 (`sk-test` 평문 0건) |
| Android 콜드 스타트 후 복호화 | ✅ OpenAI 가 마스킹된 키를 에코 → 401 정상 표시 |
| iOS 시뮬레이터 빌드 | ✅ (wry 패치 후) |
| iOS 시뮬레이터 실행 | ✅ 3탭 렌더 |
| iOS Keychain 저장/조회 | ⚠️ **미검증** — 시뮬레이터 UI 자동화에 macOS 보조기능 권한 필요 |

## 남은 일

- [ ] `gh auth login` 후 private 레포 생성 + 푸시 (사용자 계정 필요)
- [ ] iOS Keychain 실동작 확인 (시뮬레이터에서 수동 1회면 충분)
- [ ] 서명 시크릿 등록 (Apple Developer / Play Console 계정 필요)
