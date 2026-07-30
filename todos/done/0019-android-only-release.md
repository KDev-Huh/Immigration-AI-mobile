# Task 0019 — Android-only GitHub Release

- **스펙**: none (사용자 직접 요청)
- **상태**: done
- **의존**: 0012, 0018

## 설명

GitHub Release 배포 워크플로에서 iOS job 을 제거하고 Android APK/AAB 생성 및 Release 업로드만 남긴다. README 도 사용자 설치/배포 기준을 Android-only 로 정리한다.

## 수정 파일 (실제 경로)

- `.github/workflows/mobile-build.yml`
- `README.md`
- `todos/BACKLOG.md`

## 구현 노트

- `CLAUDE.md` 보안 전제는 유지한다.
- iOS 개발 스크립트는 코드에서 삭제하지 않는다. Release workflow 에서만 제외한다.
- Android Release asset 업로드 경로는 유지한다.

## Acceptance Criteria

- [x] `mobile-build.yml` 에 iOS job 이 없음
- [x] 태그 push 또는 수동 release 실행 시 Android job 만 실행됨
- [x] README 가 GitHub Release 설치는 Android 전용임을 명확히 설명
- [x] YAML 파싱 및 diff 공백 검사 통과

## 테스트

- 정적: workflow YAML 파싱, `git diff --check`
- 통합: 태그 push 후 GitHub Actions 수동 확인
