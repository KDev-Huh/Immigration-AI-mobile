---
description: 승인된 태스크 구현 (Work 단계, 필요 시 TDD)
---

태스크: $ARGUMENTS  (예: todos/active/0001-document-upload.md)

Work 단계 수행:

1. 태스크 파일 + 연결된 `docs/specs/NNNN` 읽기. 상태 `in-progress`로.
2. 테스트 요구가 있으면 **테스트 먼저** 작성(실패 확인) → 구현 → 통과 (TDD).
3. "수정 파일"에 명시된 경로만 건드림. 범위 밖 변경 시 태스크에 사유 기록.
4. IPC 추가/변경 시 4곳 동기화: `commands.rs` + `lib.rs` + `ipc.ts` + `types.ts`.
5. 검증: `cd src-tauri && cargo test` / `npm test` / `cargo check`.
6. Acceptance Criteria 체크박스 갱신. 전부 통과하면 상태 `review`로.

보안 규칙 위반(유출불가 문서 클라우드 경로 진입) 발견 시 즉시 중단·보고.
