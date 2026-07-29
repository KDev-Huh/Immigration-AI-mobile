# Task 0003 — PDF 파서 이식 (pdf-extract)

- **상태**: done
- **의존**: 0002

## 구현

- `src-tauri/src/documents/parser.rs`: 데스크탑판 이식.
- 데스크탑과 차이: 입력이 **경로가 아니라 바이트**(`parse_bytes`). `parse_file` 은 저장된 원본용 래퍼.
- 페이지 경계 미보존 → 내용 기준 pseudo-page(1500자) 분할. 출처(p.N) 표시용.
- 스캔 PDF(텍스트 20자 미만)는 명시적 에러 — OCR 미지원.

## Acceptance Criteria

- [x] 비-PDF 확장자 거부 (대소문자 무시)
- [x] 손상 바이트는 panic 아니라 Err
- [x] pseudo-page 분할 시 문자 손실 없음
- [x] `cargo check --target aarch64-linux-android` / `aarch64-apple-ios` 통과 (pdf-extract 모바일 컴파일 확인)
