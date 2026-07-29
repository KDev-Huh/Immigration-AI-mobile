# Task 0005 — 클라우드 임베딩 (OpenAI text-embedding-3-small)

- **상태**: done
- **의존**: 0004

## 구현

- `src-tauri/src/rag/embedding.rs`: `EMBED_MODEL=text-embedding-3-small`, `EMBED_DIM=1536`, 배치 64.
  - 데스크탑의 `Embedder{Local,Cloud}` enum 제거 — 분기 자체가 없음.
- `src-tauri/src/llm/cloud.rs::embed`: OpenAI `/v1/embeddings` 호출.
  - **응답을 `index` 기준 정렬** — 순서가 어긋나면 청크↔벡터 대응이 깨져 출처가 틀린다.
  - 개수 불일치 시 에러.

## 결정

- 채팅 공급자가 Anthropic 이어도 **임베딩은 항상 OpenAI**. Anthropic 은 임베딩 API 가 없음.
  → `security::embedding_api_key()` 가 OpenAI 키를 강제 조회하고, 없으면 안내 메시지 반환.

## Acceptance Criteria

- [x] 응답 순서가 뒤집혀도 index 로 복원
- [x] 개수 불일치 → Err
- [x] batch_spans 연속·무중복·전체 커버
