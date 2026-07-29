# Task 0001 — 프로젝트 스캐폴드 (Tauri v2 모바일)

- **스펙**: `docs/specs/0001-scaffold.md`
- **상태**: todo
- **의존**: none

## 설명

모바일판 뼈대 구축. Tauri v2 + React/TS 프론트 + Rust 코어 모듈 경계 + IPC 계약 스텁 + 3탭(문서/채팅/설정) 골격. 로컬 LLM 관련은 일절 없음(클라우드 전용).

## 수정/생성 파일

- 루트: `package.json`, `tsconfig.json`, `vite.config.ts`, `index.html`
- `src/`: `main.tsx`, `App.tsx`(3탭), `types.ts`, `lib/ipc.ts`, `tabs/{DocumentsTab,ChatTab,SettingsTab}.tsx`
- `src-tauri/`: `Cargo.toml`, `tauri.conf.json`, `build.rs`, `src/{main,lib,commands}.rs`
- `src-tauri/src/`: `documents/`, `rag/`(단일 컬렉션 전제), `llm/cloud.rs`, `security/` 스텁

## 구현 노트

- 데스크탑 `../Immigration-AI`에서 구조 참고하되 **로컬 LLM 제외**: ollama, 로컬 임베딩, 2컬렉션 분리 없음.
- IPC는 데스크탑에서 이식 후 로컬 LLM 커맨드 제거. `ask`는 cloud 전용.
- 임베딩 차원 상수 1536(OpenAI)로.
- 스텁 본문은 `todo!()` 허용 — 계약·경계만 확정.

## Acceptance Criteria

- [ ] `npm run dev` 3탭 렌더+전환
- [ ] `npm test` vitest 1개 통과
- [ ] `cargo check` 통과
- [ ] `cargo test` 1개 통과
- [ ] 코드 전역에 ollama/로컬임베딩 없음 (grep 확인)

## 테스트

- 단위: 프론트 순수 헬퍼 1개, Rust 순수 헬퍼 1개.
