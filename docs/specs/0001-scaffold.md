# Spec 0001 — 프로젝트 스캐폴드 (Tauri v2 모바일)

- **상태**: draft
- **작성일**: 2026-07-29
- **관련 태스크**: `todos/active/0001-scaffold.md`

## 문제 / 목적

모바일판 개발을 위한 뼈대 구축. 데스크탑 코어를 이식할 수 있는 Tauri v2 모바일 프로젝트 구조 + 계약(IPC) 스텁 + 3탭 UI 골격. 이후 태스크들이 이 위에 기능을 채움.

## 범위

- 포함: Tauri v2 모바일 프로젝트 초기화, React/TS 프론트, Rust 모듈 경계(documents/rag/llm/security) 스텁, IPC 계약 스텁, 3탭(문서/채팅/설정) 빈 화면, 빌드·테스트 설정(vitest/cargo).
- 제외: 실제 파싱·임베딩·검색·채팅 로직(후속 태스크), iOS/Android 실기기 빌드(3단계).

## 동작 명세

1. `npm run dev`로 프론트 개발서버, 탭 3개 전환 확인.
2. Rust 코어가 `cargo check` 통과(스텁 `todo!()` 허용).
3. IPC 계약: `list_documents`/`upload_document`/`ask`/`set_api_key` 등 시그니처만 정의(데스크탑과 유사, 로컬 LLM 관련 제외).
4. 벡터DB는 단일 컬렉션 전제로 모듈 경계만.

## 데이터 / 계약 변경

- 신규 프로젝트라 전체가 신규. IPC는 데스크탑 `ipc.ts`에서 로컬 LLM 항목 제외하고 이식.

## Acceptance Criteria

- [ ] `npm run dev` 로 3탭 UI 렌더 + 전환
- [ ] `npm test`(vitest) 최소 1개 통과
- [ ] `cargo check` 통과 (모듈 경계 + 스텁)
- [ ] `cargo test` 최소 1개 통과 (예: 순수 헬퍼)
- [ ] 로컬 LLM/Ollama 코드 없음 (클라우드 전용 확인)

## 테스트 요구

- 단위: 프론트 순수 헬퍼 1개(vitest), Rust 순수 헬퍼 1개(cargo).

## 미확정 / 리스크

- Tauri v2 모바일 초기화 방식(`tauri android/ios init`)은 이 태스크에선 데스크탑 타깃으로 골격만, 실제 모바일 타깃 추가는 후속.
- 모바일 보안저장 크레이트 미확정 → security 모듈은 인터페이스만.
