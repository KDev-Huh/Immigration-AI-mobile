---
description: 검증 통과분 커밋 + 푸시 (Ship 단계)
---

대상: $ARGUMENTS  (커밋할 태스크/범위)

Ship 단계:

1. 테스트 재확인: `cd src-tauri && cargo test` + `npm test`. 실패 시 중단.
2. `git status` / `git diff` 로 변경 검토. 무관한 변경 섞였으면 분리.
3. 커밋: `docs/git-convention.md` 준수 — `type :: 메시지`.
   - type: feat/fix/refactor/docs/test/chore/style/perf/build/ci/revert
   - 본문에 관련 `Task NNNN` / `Spec NNNN` 참조.
4. 완료 태스크 `todos/active/` → `todos/done/` 이동 (별도 `chore ::` 또는 같은 커밋).
5. 푸시: `git push`. 기본 브랜치면 먼저 브랜치 분기.

커밋 메시지 끝에 붙임:
Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
