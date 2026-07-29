# docs/ — 프로젝트 지식 베이스 (모바일)

하네스 엔지니어링 구조. 데스크탑판(`../Immigration-AI`)과 동일 방식.

```
docs/
├── architecture.md
├── git-convention.md
├── specs/   기능 명세 (_TEMPLATE.md)
└── adr/     아키텍처 결정 (_TEMPLATE.md)
```

관련: 상위 기획 [`../기획서.md`](../기획서.md) · 개발 규칙 [`../CLAUDE.md`](../CLAUDE.md) · 태스크 [`../todos/`](../todos/)

## 워크플로 (Plan → Work → Review → Ship)

스펙 작성 → **사람 승인** → TDD 구현 → 독립 검증 → 통과 시 `todos/done/` 이동.
