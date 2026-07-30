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
| iOS Keychain 보안저장 | ✅ 저장→조회→복호화→삭제 전 경로 확인 |
| GitHub 레포 | ✅ `KDev-Huh/Immigration-AI-mobile` (**public**) |
| CI (GitHub Actions) | ✅ public 전환으로 Actions 무료 사용 |
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
5. **Android 16KB 페이지 기기에서 .so 로드 실패.** Android 15+ 의 `ps16k` 시스템 이미지와
   Pixel 8/9 이후 실기기는 페이지 크기가 16KB 다. 기본 4KB 정렬 .so 는 `dlopen` 이 거부한다
   (`empty/missing DT_HASH/DT_GNU_HASH ... new hash type from the future?` — 메시지는 해시를
   가리키지만 실제 원인은 정렬이다). 4KB 이미지에서는 멀쩡히 돌아서 놓치기 쉽다.
   `build.rs` 에서 `-Wl,-z,max-page-size=16384` 를 붙여 해결. **`.cargo/config.toml` 의
   rustflags 로는 안 된다** — tauri CLI 가 `RUSTFLAGS` 를 직접 세팅해 config 값을 덮는다.
6. **`tauri ios dev` 는 기기를 지정하지 않으면 연결된 실기기를 고른다** → 서명 요구로 실패.
   시뮬레이터를 이름으로 넘기면 `-sdk iphonesimulator` 로 빌드돼 서명이 생략된다.
   `npm run ios:sim` 이 이걸 처리한다.
7. **포트 1420 이 점유돼 있으면** vite(strictPort) 가 죽고, 그 결과 Xcode 의 Rust 빌드 단계가
   `failed to read CLI options: ... ConnectionRefused` 라는 무관해 보이는 패닉을 낸다.
   원인은 남아 있는 dev 서버다. `ios:sim` 이 미리 잡아준다.

## 하네스 워크플로

`docs/specs/` 스펙 → 사람 승인 → `/harness-work` → `/harness-review` → `todos/done/` +
커밋(`type :: 메시지`). 진행 상황은 `todos/BACKLOG.md`.

## 다음 할 일

1. 실제 OpenAI 키로 문서 업로드 → 인덱싱 → 질의 E2E (유일하게 남은 기능 검증)
2. 서명 시크릿 등록 후 `mobile-build.yml` 릴리즈 실행 (Apple Developer / Play Console 계정)
