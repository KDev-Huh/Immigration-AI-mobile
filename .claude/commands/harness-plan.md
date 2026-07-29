---
description: 스펙 작성 + 태스크 분해 (Plan 단계)
---

기능 요청: $ARGUMENTS

Plan 단계 수행:

1. 실제 코드베이스를 먼저 스캔 (Grep/Glob/Read). 영향받는 파일·심볼을 실제 경로로 식별.
2. `docs/specs/_TEMPLATE.md` 형식으로 `docs/specs/NNNN-<slug>.md` 작성. NNNN은 기존 최대+1.
3. Acceptance Criteria는 관찰 가능·검증 가능하게. 보안 규칙(유출불가 미노출) 항목 포함.
4. 스펙을 의존성 있는 태스크로 쪼개 `todos/active/NNNN-*.md` (템플릿 `todos/_TEMPLATE.md`). 수정 파일은 실제 경로로.
5. `todos/BACKLOG.md` 갱신.
6. **여기서 멈추고** 스펙·태스크를 사람에게 검토 요청. 승인 전 구현 금지.

규칙은 `CLAUDE.md` 준수. 근거 없는 추측 금지 — 코드 확인.
