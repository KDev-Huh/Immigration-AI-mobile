# 아키텍처 개요 (모바일)

전체는 [`../CLAUDE.md`](../CLAUDE.md) 참고. 이 문서는 요약 + 결정 링크.

## 계층

| 계층 | 위치 | 책임 |
|---|---|---|
| UI | `src/` (React/TS) | 모바일 반응형 탭, IPC 호출만 |
| IPC | `src/lib/ipc.ts` ↔ `src-tauri/src/commands.rs` | 1:1 |
| 도메인 | `src-tauri/src/{documents,rag,llm,security}` | 로직 |
| 저장 | 기기 로컬 (벡터DB + 보안저장) | 원격 없음 |

## 데스크탑과의 핵심 차이

- 로컬 LLM(Ollama) 제거 → 임베딩·생성 모두 클라우드.
- 벡터DB 단일 컬렉션 (유출가능 전용, 2컬렉션 분리 불필요).
- 임베딩 차원 1536 (OpenAI text-embedding-3-small).

## 결정 기록 (ADR)

- [0001 — 기술 스택 (Tauri 모바일 · 클라우드 전용)](adr/0001-tech-stack.md)
- [0002 — 클라우드 전용 보안 모델 (유출가능만)](adr/0002-cloud-only-security.md)
