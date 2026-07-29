# 비자 행정사 AI — 모바일판

비자 행정사용 RAG 챗봇의 **iOS/Android 앱**. 업로드한 문서에 근거해서만 답하고, 근거가 없으면
“자료 없음”이라고 답한다. 데스크탑판은 [`../Immigration-AI`](../Immigration-AI).

## 보안 전제 (반드시 읽을 것)

모바일은 임베딩·생성 모두 클라우드로 나간다. 따라서:

- **유출가능(leakable) 문서만 업로드할 수 있다.** 유출불가 문서는 Rust 가 거부한다.
  유출불가 문서를 다뤄야 하면 로컬 LLM 을 쓰는 데스크탑판을 사용할 것.
- API 키는 iOS Keychain / Android Keystore 에만 저장한다. 평문 파일 저장 경로는 없다.
- 문서·벡터는 기기 로컬에만 남는다. 원격 서버·텔레메트리 없음.

## 구조

```
src/                     React/TS — 하단 3탭 (문서 / 채팅 / 설정)
  lib/ipc.ts             ↕ 1:1
src-tauri/src/
  commands.rs            IPC 진입점. 업로드 거부 판정 등 검증·라우팅
  documents/             파싱(pdf-extract) · 청킹 · 메타/원본 저장
  rag/                   클라우드 임베딩 · 단일 컬렉션 벡터DB · 하이브리드 검색
  llm/cloud.rs           OpenAI / Anthropic HTTP
  security/store/        보안저장 (ios.rs / android.rs / desktop.rs)
  vendor/wry             wry iOS 크래시 우회 패치 (docs/adr/0003)
```

데스크탑판과의 결정적 차이는 **로컬 LLM 경로가 아예 없다**는 것. `collection_for(backend)`
같은 보안 분기도 없다 — 컬렉션이 하나뿐이라 잘못 고를 여지가 구조적으로 없다.

## 필요한 것

**OpenAI API 키가 필수다.** 채팅을 Anthropic 으로 하더라도 임베딩(문서 색인·검색)은 항상
OpenAI 를 쓴다 — Anthropic 에는 임베딩 API 가 없다.
ChatGPT Plus 구독이나 Codex OAuth 로는 대체할 수 없다(임베딩 스코프 없음).

## 개발

```bash
npm install
npm run dev            # 브라우저에서 UI 만 (IPC 호출은 실패)
npm run tauri:dev      # 데스크탑 창으로 전체 앱

npm test               # vitest
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
```

### 모바일

```bash
export ANDROID_HOME=$HOME/Library/Android/sdk
export NDK_HOME=$ANDROID_HOME/ndk/<버전>

npm run android:init && npm run android:dev
npm run ios:init && npm run ios:dev
```

`src-tauri/gen/` 은 커밋하지 않는다. CI 도 매번 `init` 으로 재생성한다.

시뮬레이터/에뮬레이터 빌드는 서명이 필요 없다:

```bash
npm run tauri android build -- --debug --apk --target aarch64
npm run tauri ios build -- --debug --target aarch64-sim --ci
```

실기기 iOS 빌드는 Apple Developer 팀이 있어야 한다.

## CI

| 워크플로 | 하는 일 |
|---|---|
| `.github/workflows/ci.yml` | 프론트 타입체크·테스트, Rust fmt/clippy/test, Android·iOS 타깃 컴파일 체크 |
| `.github/workflows/mobile-build.yml` | Android APK/AAB, iOS 시뮬레이터 앱. 서명 시크릿이 있으면 서명 빌드까지 |

서명 시크릿(미설정 시 해당 스텝은 건너뜀):
`ANDROID_KEY_BASE64` · `ANDROID_KEY_ALIAS` · `ANDROID_KEY_PASSWORD` ·
`APPLE_CERTIFICATE` · `APPLE_CERTIFICATE_PASSWORD` · `APPLE_DEVELOPMENT_TEAM` ·
`APPLE_API_KEY` · `APPLE_API_ISSUER` · `APPLE_API_KEY_PATH`

## 문서

- `기획서.md` — 상세 기획
- `CLAUDE.md` — 에이전트 개발 규칙
- `docs/architecture.md`, `docs/adr/` — 구조·결정 기록
- `todos/BACKLOG.md` — 진행 상황
