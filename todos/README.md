# todos/ — 태스크 (모바일)

스펙(`docs/specs/`)을 실행 단위로 분해. 코드와 함께 버전관리.

```
todos/
├── BACKLOG.md      우선순위 대기열
├── _TEMPLATE.md
├── active/         진행 중 (WIP 최소)
└── done/           완료
```

규칙: 태스크=파일 하나(`NNNN-kebab.md`). 완료 시 acceptance 통과 + 테스트 통과 → `done/`.
