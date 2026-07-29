# Task 0007 — 인덱싱 파이프라인

- **상태**: done
- **의존**: 0006

## 구현

- `src-tauri/src/commands.rs::index_document` → `run_index`:
  1. **키 선확인** (무거운 파싱 후 키 없음으로 실패하면 낭비)
  2. 파싱 — `spawn_blocking` (pdf-extract 는 CPU 바운드, 런타임 블로킹 방지)
  3. 청킹
  4. 클라우드 임베딩 — 진행률 0.2~0.95 구간 매핑
  5. `jobs::build_records` → 벡터DB upsert
  6. `mark_ready`
- 실패 시 `mark_error` 로 문서를 남겨 **재시도 가능**하게 함. 프론트에 "재시도" 버튼.
- 진행률은 Tauri event `index-progress` 로 emit → `DocumentsTab` 이 구독.
- Mutex 가드는 항상 스코프로 감싸 `await` 를 넘기지 않음 (Send 위반 방지).

## Acceptance Criteria

- [x] 청크·벡터 개수 불일치 → Err
- [x] 임베딩 차원 != 1536 → Err (로컬 임베딩 1024 벡터 혼입 방지)
- [x] 진행률 event 페이로드가 types.ts `IndexProgress` 와 일치
