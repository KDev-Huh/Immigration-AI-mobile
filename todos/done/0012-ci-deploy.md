# Task 0012 — GitHub 레포 + Actions CI/빌드 (iOS/Android)

- **상태**: done (배포는 계정 준비 대기)
- **의존**: 0011

## 완료

- 레포: `KDev-Huh/Immigration-AI-mobile` (**private**), 기본 브랜치 `master`, 146 파일 푸시.
- `.github/workflows/ci.yml`
  - 프론트: typecheck + vitest + vite build
  - Rust: fmt / clippy `-D warnings` / test (ubuntu + webkit2gtk 등 시스템 의존성)
  - 모바일 타깃 컴파일 체크: `aarch64-linux-android`, `aarch64-apple-ios`
    → cfg 분기된 보안저장 구현(JNI / Keychain)이 실제로 빌드되는지 CI 가 보장
- `.github/workflows/mobile-build.yml`
  - Android: `tauri android init` → APK(+태그 시 AAB)
  - iOS: `tauri ios init` → 시뮬레이터 앱(서명 불필요) 아티팩트
  - 서명은 시크릿이 있을 때만. 시크릿 게이팅은 **잡 레벨 env** 로 올렸다
    — 스텝 자신의 env 는 그 스텝 `if:` 평가 시점에 존재하지 않는다.
  - `gen/` 은 커밋하지 않으므로 CI 가 매번 재생성

## 로컬 검증 결과

| 항목 | 결과 |
|---|---|
| Android APK 빌드 (aarch64) | ✅ |
| Android 에뮬레이터 3탭 실행 | ✅ |
| Android Keystore 저장 | ✅ prefs 에 암호문만 (`sk-test` 평문 0건) |
| Android 콜드 스타트 후 복호화 | ✅ OpenAI 가 마스킹된 키 에코 → 401 정상 표시 |
| iOS 시뮬레이터 빌드·실행 | ✅ (wry 패치 후 — ADR 0003) |
| iOS Keychain 저장/조회 | ⚠️ 미검증 — 시뮬레이터 UI 자동화에 macOS 보조기능 권한 필요 |

## CI 첫 실행 결과

세 잡 모두 **시작 전 거부**:

```
The job was not started because recent account payments have failed
or your spending limit needs to be increased.
```

워크플로 문법 문제가 아니다(잡 3개가 정상 파싱·열거됨). 계정 결제/한도 문제라
사용자 조치가 필요하다. private 레포는 Actions 분을 소모하며, `macos-latest` 는
분당 10배로 계산된다.

## 남은 일 (계정 필요)

- [ ] GitHub Billing 해결 → CI 재실행
- [ ] iOS Keychain 실동작 확인 (시뮬레이터 수동 1회)
- [ ] 실제 OpenAI 키로 업로드→인덱싱→질의 E2E
- [ ] 서명 시크릿 등록 → 스토어 배포
