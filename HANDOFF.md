# 프로젝트 핸드오프 — Immigration-AI-mobile

> 새 세션은 이 문서 → `README.md` → `CLAUDE.md` → `기획서.md` 순으로 보면 된다.

## 한 줄 요약

비자 행정사용 **RAG 챗봇의 모바일판**(iOS/Android, Tauri v2). 데스크탑판
(`../Immigration-AI`)에서 파생했고 **로컬 LLM 을 제거해 클라우드 전용**으로 만들었다.

## 현재 상태 (2026-07-29)

**기능 구현 완료. 양 플랫폼 실행 검증됨. 남은 건 레포 생성과 스토어 서명뿐.**

| 항목 | 상태 |
|---|---|
| Rust 코어 (문서/RAG/LLM/보안) | ✅ 51 테스트 통과, clippy `-D warnings` 클린 |
| 프론트 3탭 | ✅ vitest 통과, Android 에뮬레이터에서 3탭 전부 렌더 확인 |
| Android 빌드·실행 | ✅ APK 빌드 → 에뮬레이터 실행 |
| Android Keystore 보안저장 | ✅ 암호문만 저장, 콜드 스타트 후 복호화까지 확인 |
| iOS 빌드·실행 | ✅ 시뮬레이터 빌드·실행 (wry 패치 필요했음 — ADR 0003) |
| iOS Keychain 보안저장 | ⚠️ 컴파일·실행만 확인. **저장/조회 실동작 미검증** |
| GitHub 레포 | ✅ `KDev-Huh/Immigration-AI-mobile` (private) |
| CI (GitHub Actions) | ⚠️ 워크플로 정상, **계정 결제 문제로 잡 시작 거부** |
| 스토어 서명·배포 | ❌ Apple Developer / Play Console 계정 필요 |

## 확정된 핵심 결정

| 항목 | 결정 | 근거 |
|---|---|---|
| 플랫폼 | Tauri v2 모바일 | Rust/React 코어 재사용 — `docs/adr/0001` |
| LLM | 클라우드 전용 (OpenAI/Anthropic) | 모바일은 로컬 LLM 비현실적 |
| 임베딩 | OpenAI `text-embedding-3-small`(1536d) | Anthropic 은 임베딩 API 부재 |
| 보안 | 유출가능 문서만 허용 | 모바일=클라우드 전송 전제 — `docs/adr/0002` |
| 벡터DB | 단일 컬렉션 | 분기가 없으면 잘못 고를 수 없다 |
| 키 저장 | iOS Keychain / Android Keystore | 평문 폴백 경로 없음 |
| wry | 벤더 패치 | iOS 실행 즉시 크래시 회피 — `docs/adr/0003` |

## 반복 질문 방지

- **ChatGPT Plus 구독 / Codex OAuth 로는 이 앱을 못 쓴다.** Codex OAuth 는 코딩 에이전트
  전용 스코프이고 **임베딩 API 가 없다.** RAG 에 임베딩은 필수 → **OpenAI API 키가 유일한
  정식 경로.** 브라우저 로그인 우회는 약관·정지 위험 때문에 하지 않는다.
- 채팅을 Anthropic 으로 해도 **OpenAI 키는 반드시 필요**하다(임베딩 때문).

## 개발 중 발견한 함정 (재발 방지)

1. **Tauri v2 는 `ndk-context` 를 초기화하지 않는다.** Android 에서 JNI 를 쓰려면
   `wry::prelude::dispatch` 로 JavaVM/Activity 를 직접 잡아야 한다. `ndk_context::android_context()`
   를 부르면 패닉 → non-unwinding abort 로 앱이 죽는다.
2. **wry 0.55.1 은 iOS 에서 무조건 크래시한다.** `platform_webview_version()` 의
   `bundle.unload()`. `vendor/wry` 패치로 회피 중 — 업스트림 수정 시 제거할 것.
3. **Android 파일 피커는 `content://` URI 를 준다.** Rust 가 경로로 못 읽으므로
   프론트에서 `plugin-fs.readFile()` 로 바이트를 읽어 IPC 로 넘긴다.
4. **보안저장 커맨드는 반드시 `spawn_blocking` 위에서.** Android 구현이 메인 스레드 왕복을
   기다리므로 메인 스레드에서 부르면 교착한다.
5. 시뮬레이터/에뮬레이터 빌드는 **서명이 필요 없다** — 실기기 iOS 만 Apple 팀이 필요.

## 하네스 워크플로

`docs/specs/` 스펙 → 사람 승인 → `/harness-work` → `/harness-review` → `todos/done/` +
커밋(`type :: 메시지`). 진행 상황은 `todos/BACKLOG.md`.

## 다음 할 일

1. GitHub Billing 해결 → CI 재실행 (`gh run rerun --failed`). private 레포는 Actions 분을 소모하고 `macos-latest` 는 10배로 계산된다.
2. iOS 시뮬레이터에서 설정 탭 → 키 저장 → 앱 재시작 → 채팅 1회
   (Android 에서 한 것과 동일한 확인. 401 이 뜨면 Keychain 왕복 정상)
3. 실제 OpenAI 키로 문서 업로드 → 인덱싱 → 질의 E2E
4. 서명 시크릿 등록 후 `mobile-build.yml` 릴리즈 실행
