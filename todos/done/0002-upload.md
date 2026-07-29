# Task 0002 — 문서 업로드(유출가능 전용) + 목록/삭제

- **상태**: done
- **의존**: 0001

## 설명

모바일 파일 피커 → 프론트에서 바이트 읽기 → IPC 로 Rust 전달 → 앱 데이터에 복사·소유.

## 구현

- `src/tabs/DocumentsTab.tsx`: `plugin-dialog.open()` + `plugin-fs.readFile()` → `uploadDocument(filename, bytes, "leakable")`.
  - Android 파일 피커는 `content://` URI 를 돌려주므로 Rust 가 경로로 직접 못 읽는다 → 바이트 전달 방식 채택.
- `src-tauri/src/documents/store.rs`: `DocStore` 이식 + 원본 바이트를 `{app_data}/files/{id}.pdf` 로 복사.
- `src-tauri/src/commands.rs`: `upload_document` 에서 **유출불가 거부**(`Sensitivity::allows_upload`), 확장자 검사, SHA-256 중복 거부.

## Acceptance Criteria

- [x] 유출불가 태그 업로드 시 Rust 가 거부 (`UPLOAD_REJECT`)
- [x] 중복(동일 해시) 업로드 거부
- [x] 삭제 시 메타 + 원본 파일 + 벡터 레코드 모두 제거
- [x] `cargo test` documents::store 5개 통과
