# Task 0006 — 벡터DB 단일 컬렉션 + 하이브리드 검색 이식

- **상태**: done
- **의존**: 0005

## 구현

- `src-tauri/src/rag/vectordb.rs`: 데스크탑판에서 **컬렉션 개념 자체를 제거**.
  - 데스크탑: `HashMap<String, Vec<Record>>` + `collection_for(backend)` 보안 분기
  - 모바일: `Vec<Record>` 단일. 유출가능 문서만 존재하므로 분기가 불필요 = 실수할 여지 없음.
  - 파일명 `leakable.json` 로 저장소 성격을 코드에 남김.
- 하이브리드 점수 `(1-λ)*cosine + λ*lexical`, λ=0.4.
- 어휘 매칭은 공백 제거 후 비교 — 한국어 띄어쓰기 편차 대응("부모초청" ↔ "부모 초청").

## Acceptance Criteria

- [x] 벡터상 더 가까운 청크라도 정확 용어 포함 청크가 상위로 역전 (λ 높을 때)
- [x] 재인덱싱 시 동일 doc_id 레코드 교체(중복 없음)
- [x] 재시작 후 영속
