# Task 0018 — Gemini API + 아이콘 + 사용자 README

- **스펙**: none (사용자 직접 요청)
- **상태**: done
- **의존**: none

## 설명

Gemini API 를 채팅 공급자로 추가하고, 앱 아이콘을 제공 이미지로 교체한다. README 는 기존 프로젝트 배경을 몰라도 설치·설정·사용·릴리즈 흐름을 이해할 수 있게 재작성한다.

## 수정 파일 (실제 경로)

- `src-tauri/src/security/mod.rs`
- `src-tauri/src/llm/cloud.rs`
- `src/types.ts`
- `src/lib/settings.ts`
- `src/lib/settings.test.ts`
- `src/tabs/ChatTab.tsx`
- `src/tabs/SettingsTab.tsx`
- `src-tauri/icons/*`
- `README.md`

## 구현 노트

- `CLAUDE.md` 기준: 모바일은 클라우드 전용, 임베딩은 항상 OpenAI, API 키는 보안저장만 사용.
- Gemini 는 채팅 생성 공급자로만 추가한다. 문서 색인·검색 임베딩은 계속 OpenAI `text-embedding-3-small`.
- Gemini REST 는 공식 문서 기준 `generateContent` + `x-goog-api-key` + `system_instruction` 형식을 사용한다.
- 새 IPC 커맨드는 없고, 기존 `CloudProvider` 계약을 확장한다.

## Acceptance Criteria

- [x] 설정 탭에서 Gemini API 키 저장/삭제 및 모델 선택 가능
- [x] 채팅 탭에서 Gemini 공급자를 선택하면 Gemini REST 생성 호출
- [x] OpenAI 키는 임베딩 필수 조건으로 유지
- [x] 제공 이미지가 Tauri 앱 아이콘으로 반영
- [x] README 만 보고 설치·키 설정·문서 업로드·채팅·Release 설치 흐름을 알 수 있음
- [x] 테스트 통과 (`npm run typecheck`, `npm test`, `npm run build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`)

## 테스트

- 단위: provider serde, Gemini 요청/응답 파싱, 설정 provider 판정
- 통합: 실제 API 키 E2E 는 키가 필요하므로 수동 검증 항목으로 남김
