# Task 0008 — 채팅 RAG

- **상태**: done
- **의존**: 0007

## 구현

- `src-tauri/src/commands.rs::ask(query, provider, model)` — **`backend` 인자 없음**(데스크탑과 차이).
  1. 쿼리 임베딩 — 클라우드(OpenAI). 데스크탑은 로컬이었음.
  2. 하이브리드 검색 (단일 컬렉션)
  3. 근거 부족(최고점수 < 0.2) → 생성 생략하고 `NO_EVIDENCE` 반환. 지어내지 않음.
  4. 컨텍스트 조립 + 출처 생성 (`retriever::assemble`, 8000자 예산)
  5. 클라우드 생성 (OpenAI/Anthropic)
- 프론트 `ChatPane` 이 답변 아래 출처(파일명·페이지·발췌)를 표시 — 컨벤션 요구사항.

## Acceptance Criteria

- [x] 근거 없으면 "자료 없음" + 빈 citations
- [x] 컨텍스트 예산 초과 시 잘라내되 최소 1개 보장
- [x] 답변에 출처(파일명·페이지) 노출
