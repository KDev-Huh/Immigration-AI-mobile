# Task 0020 — Android 업로드 거부 + 런처 아이콘 미반영 수정

- **상태**: done
- **의존**: 0018, 0019

## 증상

1. Android 실기기에서 PDF 를 골라도 `오류: 지원하지 않는 포맷: .(현재 PDF만 지원)`.
2. 설치한 앱의 런처 아이콘이 지정한 이미지가 아니라 Tauri 기본 아이콘.

## 원인 1 — 확장자 기반 포맷 판정

에러 메시지의 확장자가 **비어 있는 것**이 단서였다.

Android 파일 피커는 `content://` URI 를 돌려주는데, 제공자에 따라 마지막 경로 조각이
파일명이 아니라 문서 ID 다.

| 제공자 | URI 마지막 조각 | 결과 |
|---|---|---|
| externalstorage | `primary:Download/answer.pdf` | `.pdf` → 통과 |
| **downloads** | `msf:1000000123` | 확장자 없음 → **거부** |

그래서 "어떤 PDF 는 되고 어떤 PDF 는 안 되는" 것처럼 보였다.

**수정**: 업로드 시점에 바이트를 이미 갖고 있으므로 확장자가 아니라 **내용(`%PDF-` 시그니처)**
으로 판정한다. 파일명은 표시·출처용으로만 쓰고, 문서 ID 형태면 날짜 기반 이름으로 대체한다.

- `parser::detect_pdf_bytes()` 신설, `detect_pdf()`(확장자 검사) 제거
- `parser::sanitize_filename()` — 경로/URI 조각 제거, `.pdf` 보장 (최종 방어선)
- 프론트 `displayName()` — 실제 파일명이 있으면 유지, ID 형태면 `문서-YYYYMMDD-HHmm.pdf`

## 원인 2 — CI 가 아이콘을 재생성하지 않음

`gen/` 은 커밋하지 않아 CI 가 매번 `tauri android init` 으로 만든다. 그런데 워크플로에
`tauri icon` 단계가 없었다. 로컬에는 아이콘이 반영돼 있어 눈치채기 어려웠다.

해시로 입증:

| 단계 | `mipmap-xxxhdpi/ic_launcher.png` |
|---|---|
| 아이콘 반영된 로컬 | `036ddb20…` |
| `android init` 재생성 직후 | `dae1ff05…` ← 기본 아이콘 |
| `tauri icon` 실행 후 | `036ddb20…` |

**수정**:
- 원본을 `아이콘.png`(루트) → `assets/app-icon.png` 로 이동 (CI 에서 비ASCII 경로 회피 + 루트 정리)
- `npm run icon` 스크립트 추가
- 워크플로에 `android init` **다음** 아이콘 반영 단계 추가
- init 직후 기본 아이콘 해시를 기록해두고, 반영 후에도 같으면 **빌드 실패**시켜 회귀를 막음

## 검증

- `cargo test` 59, `npm test` 8 (신규: 시그니처 판정·헤드 범위·파일명 정리·URI 파싱)
- Android 에뮬레이터 실기기 흐름: Downloads 제공자(=실패하던 경로)에서 PDF 선택 →
  **업로드 성공**, 목록에 `testdoc.pdf` 등록. 이후 단계 에러는 OpenAI 키 미등록 건으로 별개.
- APK 내부 `res/mipmap-xxxhdpi-v4/ic_launcher.png` 해시가 `036ddb20…`(사용자 아이콘)와 일치,
  런처에도 정상 표시 확인.
