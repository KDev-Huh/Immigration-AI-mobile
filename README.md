# 비자 행정사 AI 모바일

업로드한 비자/체류 관련 PDF 문서를 근거로 답변하는 iOS/Android용 RAG 챗봇입니다.
문서에 근거가 없으면 답을 지어내지 않고 "자료 없음"으로 응답합니다.

## 한눈에 보기

- 대상: 비자 행정 업무 중 문서 근거를 빠르게 확인하려는 사용자
- 플랫폼: Android, iOS
- 지원 파일: 텍스트가 추출되는 PDF
- 채팅 모델: OpenAI, Anthropic, Google Gemini 중 선택
- 임베딩/검색: 항상 OpenAI API 사용
- 저장 위치: 문서, 벡터, API 키 모두 사용자 기기 로컬
- 보안 규칙: 유출가능 문서만 업로드 가능

## 반드시 알아야 할 점

이 앱은 모바일 기기에서 로컬 LLM을 실행하지 않습니다. 문서 색인과 답변 생성을 위해 클라우드 API를
사용합니다.

그래서 유출불가 문서는 업로드하면 안 됩니다. 앱도 `confidential` 문서 업로드를 Rust 쪽에서 거부합니다.
유출불가 문서를 다뤄야 한다면 로컬 LLM을 사용하는 데스크탑판을 사용해야 합니다.

OpenAI API 키는 필수입니다. 채팅 모델을 Anthropic이나 Gemini로 선택해도, 문서 인덱싱과 검색용
임베딩은 OpenAI `text-embedding-3-small`을 사용합니다.

## 사용 준비

1. Android APK를 설치합니다.
2. 앱을 열고 `설정` 탭으로 이동합니다.
3. OpenAI API 키를 저장합니다.
4. 채팅 공급자를 선택합니다.
5. 선택한 채팅 공급자의 API 키도 저장합니다.
6. `문서` 탭에서 PDF를 업로드합니다.
7. 인덱싱이 완료되면 `채팅` 탭에서 질문합니다.

### API 키

| 목적 | 필요 키 |
|---|---|
| 문서 업로드 후 인덱싱 | OpenAI API 키 필수 |
| OpenAI로 채팅 | OpenAI API 키 |
| Anthropic으로 채팅 | OpenAI API 키 + Anthropic API 키 |
| Gemini로 채팅 | OpenAI API 키 + Gemini API 키 |

API 키는 iOS Keychain 또는 Android Keystore 기반 보안저장에 저장됩니다. 프론트엔드 localStorage나
평문 파일에는 저장하지 않습니다.

## Android 설치

GitHub Releases에서 최신 APK를 내려받아 설치합니다.

1. 이 저장소의 `Releases` 페이지를 엽니다.
2. 최신 버전에서 `immigration-ai-mobile-...-android-...apk` 파일을 다운로드합니다.
3. Android가 "알 수 없는 앱 설치" 권한을 요구하면 현재 브라우저 또는 파일 앱에만 허용합니다.
4. 설치 후 앱을 실행합니다.

`debug.apk`도 직접 설치할 수 있습니다. 다만 배포/운영용으로는 `ANDROID_*` 서명 시크릿으로 생성한
`release.apk` 사용을 권장합니다.

## GitHub Release 배포

태그를 push하면 GitHub Actions가 Android APK를 빌드하고 Release asset으로 업로드합니다.

```bash
git tag v0.1.0
git push origin v0.1.0
```

빌드 결과:

- 서명 시크릿 있음: `immigration-ai-mobile-v0.1.0-android-release.apk`
- 서명 시크릿 없음: `immigration-ai-mobile-v0.1.0-android-debug.apk`
- 서명 시크릿 있음 + 릴리즈 빌드: Play Console용 `.aab`도 함께 생성

수동으로 실행하려면 GitHub Actions에서 `모바일 빌드` 워크플로를 선택하고:

- `release`: `true`
- `tag`: `v0.1.0` 같은 릴리즈 태그

를 입력합니다.

### Android 서명 시크릿

운영 배포용 APK/AAB를 만들려면 저장소 Secrets에 아래 값을 등록합니다.

| Secret | 의미 |
|---|---|
| `ANDROID_KEY_BASE64` | 업로드 키스토어 `.jks` 파일을 base64 인코딩한 값 |
| `ANDROID_KEY_ALIAS` | 키 alias |
| `ANDROID_KEY_PASSWORD` | 키스토어/키 비밀번호 |

## 개발 실행

```bash
npm install
npm run dev
```

브라우저에서 UI만 확인할 때 사용합니다. 이 모드에서는 Tauri IPC가 없으므로 문서 업로드, 키 저장,
채팅 호출 같은 기능은 실패할 수 있습니다.

전체 앱을 데스크탑 창으로 확인하려면:

```bash
npm run tauri:dev
```

## 모바일 개발

Android:

```bash
export ANDROID_HOME=$HOME/Library/Android/sdk
export NDK_HOME=$ANDROID_HOME/ndk/<버전>

npm run android:init
npm run android:dev
```

iOS 시뮬레이터:

```bash
npm run ios:init
npm run ios:sim
```

`tauri ios dev`를 직접 실행하면 연결된 실기기를 선택해 서명 오류가 날 수 있습니다. 시뮬레이터 실행은
`npm run ios:sim`을 사용하세요.

`src-tauri/gen/`은 생성 산출물입니다. 커밋하지 않습니다.

## 검증 명령

프론트엔드:

```bash
npm run typecheck
npm test
npm run build
```

Rust/Tauri:

```bash
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## 구조

```text
src/
  App.tsx                  하단 3탭 앱 셸
  tabs/DocumentsTab.tsx    PDF 업로드, 인덱싱 진행률
  tabs/ChatTab.tsx         RAG 채팅
  tabs/SettingsTab.tsx     모델 선택, API 키 저장, 글자 크기
  lib/ipc.ts               Tauri IPC 래퍼

src-tauri/src/
  commands.rs              IPC 진입점, 업로드 검증, RAG 라우팅
  documents/               PDF 파싱, 청킹, 문서 저장
  rag/                     OpenAI 임베딩, 벡터DB, 하이브리드 검색
  llm/cloud.rs             OpenAI, Anthropic, Gemini 생성 호출
  security/                iOS Keychain, Android Keystore, 데스크탑 키체인
```

## 현재 한계

- 스캔 PDF/OCR은 지원하지 않습니다.
- OpenAI API 키 없이 문서 인덱싱과 검색을 할 수 없습니다.
- 유출불가 문서는 모바일 앱에서 사용할 수 없습니다.
- 실제 스토어 배포는 Apple Developer 또는 Play Console 계정 설정이 필요합니다.

## 참고 문서

- `CLAUDE.md`: 개발 규칙과 보안 전제
- `HANDOFF.md`: 현재 프로젝트 상태와 남은 작업
- `docs/architecture.md`: 아키텍처 요약
- `docs/adr/`: 결정 기록
- `todos/BACKLOG.md`: 작업 목록
