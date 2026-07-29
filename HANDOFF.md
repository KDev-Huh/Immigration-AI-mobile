# 프로젝트 핸드오프 — Immigration-AI-mobile

> 새 세션이 이 프로젝트를 빠르게 이해하기 위한 문서. 먼저 이걸 읽고 `CLAUDE.md` → `기획서.md` 순으로 보면 됨.

## 한 줄 요약

비자 행정사용 **RAG 챗봇의 모바일판**. 데스크탑판(`../Immigration-AI`)에서 파생했고, **로컬 LLM을 제거하고 클라우드 LLM 전용**으로 만든다. (iOS/Android, Tauri v2 모바일)

## 데스크탑판과의 관계

- 형제 폴더 `../Immigration-AI` = 완성된 데스크탑 앱 (Tauri, 로컬 LLM(Ollama) + 클라우드 겸용, 배포까지 됨).
- 이 모바일판은 데스크탑의 **코어 로직을 이식**하되 클라우드 전용으로 단순화.
- **재사용**: `documents`(parser=pdf-extract, chunker), `rag`(retriever, 하이브리드 검색, 프롬프트), ChatPane 채팅 UI 패턴.
- **제거**: `llm/ollama`, 로컬 임베딩, 벡터DB 2컬렉션(all/leakable) 분리, Ollama 관리 탭.

## 확정된 핵심 결정 (ADR 참고)

| 항목 | 결정 | 근거 |
|---|---|---|
| 플랫폼 | Tauri v2 모바일 (iOS/Android) | Rust/React 코어 재사용 — `docs/adr/0001` |
| LLM | **클라우드 전용** (OpenAI API, Anthropic 선택) | 모바일은 로컬 LLM 비현실적 |
| 임베딩 | **클라우드** OpenAI `text-embedding-3-small`(1536d) | 로컬 임베딩 없음 |
| 보안 | **유출가능(leakable) 문서만 허용**, 유출불가 업로드 금지 | 모바일=클라우드 전송 전제 — `docs/adr/0002` |
| 벡터DB | 단일 컬렉션(유출가능 전용), 기기 로컬 | 2컬렉션 분리 불필요 |
| API 키 | 모바일 보안저장(iOS Keychain/Android Keystore) | 평문 금지 |

## 중요 제약 (반복 질문 방지)

- **ChatGPT Plus 구독 / Codex OAuth 로 앱에서 GPT 못 씀.** Codex OAuth는 Codex(코딩 에이전트) 전용 스코프이고, **임베딩 API가 없음.** RAG엔 임베딩 필수 → **OpenAI API 키가 유일한 정식 경로.** (약관·정지 위험 때문에 브라우저 로그인/세션 우회 안 함.)

## 현재 상태 (2026-07-29)

**하네스 구조 + 기획만 완료. 코드는 아직 없음(스캐폴드 전).**

```
Immigration-AI-mobile/
├── 기획서.md            상세 기획
├── CLAUDE.md            에이전트 개발 규칙(모바일)
├── HANDOFF.md           (이 문서)
├── docs/
│   ├── architecture.md
│   ├── specs/0001-scaffold.md      첫 스펙
│   └── adr/ 0001-스택, 0002-클라우드전용보안
└── todos/
    ├── BACKLOG.md                  12개 태스크 로드맵(1~3단계)
    └── active/0001-scaffold.md     ← 다음 할 일
```

- git: 초기화됨, 첫 커밋 완료. **원격 레포 아직 없음**(별도 새 레포 예정).

## 하네스 워크플로

`docs/specs/` 스펙 → **사람 승인** → `/harness-work <태스크>` 로 TDD 구현 → `/harness-review` → 통과 시 `todos/done/` 이동 + 커밋(`type :: 메시지`).

## 다음 할 일 — Task 0001

**프로젝트 스캐폴드**: Tauri v2 + React/TS 프론트 + Rust 코어 모듈 경계(documents/rag/llm/security) 스텁 + IPC 계약 + 3탭(문서/채팅/설정) 골격. **로컬 LLM 코드 전무.**
→ `todos/active/0001-scaffold.md` 참고. 승인되면 여기서부터 구현 시작.

## 시작 방법 (새 세션)

1. 이 폴더에서 세션 열면 `CLAUDE.md` 자동 로드됨.
2. 이 `HANDOFF.md` + `기획서.md` + `todos/active/0001-scaffold.md` 읽기.
3. 필요하면 데스크탑판 `../Immigration-AI/src-tauri/src/{documents,rag}/` 참고해 이식.
4. `/harness-work todos/active/0001-scaffold.md` 로 착수.
