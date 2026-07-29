# CLAUDE.md — 개발 하네스 가이드 (모바일)

비자 행정사용 RAG AI 챗봇 **모바일판**. 클라우드 LLM 전용. 상세 기획 `기획서.md`. 데스크탑판은 `../Immigration-AI`.

## 절대 규칙 (보안)

1. **유출불가 문서는 업로드 금지.** 모바일은 클라우드 전송이 전제 → 유출가능(leakable) 문서만 허용. 업로드 시 유출불가면 거부.
2. API 키는 평문 저장 금지. 모바일 보안저장(iOS Keychain/Android Keystore)만 사용.
3. 문서·벡터·키 전부 기기 로컬. 원격 서버·텔레메트리 없음.
4. `data/`, `.env`, 키 커밋 금지.

## 아키텍처

```
프론트(React/TS, src/)  ──invoke──▶  Rust(src-tauri/src/)
  탭 문서관리                         commands.rs (IPC, 검증/라우팅)
  탭 채팅 (cloud)                      ├─ documents/  파싱·청킹 (데스크탑 재사용)
  탭 설정 (키/모델)                     ├─ rag/        클라우드임베딩·벡터DB·하이브리드검색
                                       ├─ llm/        cloud (OpenAI/Anthropic)
                                       └─ security/   모바일 보안저장
```

- **로컬 LLM 없음**: Ollama·로컬임베딩·2컬렉션 분리 전부 제거. 벡터DB 단일 컬렉션.
- **임베딩·생성 모두 클라우드**: 쿼리 임베딩도 OpenAI. (데스크탑은 로컬 임베딩이었음 — 여기선 클라우드)
- **하이브리드 검색**(벡터+키워드) 데스크탑에서 이식 — 한국어 정확 용어 recall.

## 데스크탑 재사용 지침

- `../Immigration-AI/src-tauri/src/documents/{parser,chunker}.rs`, `rag/{retriever,vectordb}.rs` 로직 이식.
- 단, `vectordb`는 컬렉션 1개로 단순화, 임베딩 차원 1536(OpenAI)로.
- `llm/ollama.rs`, 로컬 임베딩 경로는 이식하지 않음.

## 하네스 워크플로 (Plan → Work → Review → Ship)

| 폴더 | 용도 |
|---|---|
| `docs/specs/` | 기능 명세 (`_TEMPLATE.md`) |
| `docs/adr/` | 아키텍처 결정 |
| `todos/active/` · `todos/done/` | 태스크 |
| `todos/BACKLOG.md` | 우선순위 대기열 |

커맨드: `/harness-plan` → `/harness-work` → `/harness-review` → `/harness-ship`. 커밋: `type :: 메시지`.

## 컨벤션

- 주석·UI 문자열 한국어. 식별자 영어.
- 새 IPC: `commands.rs` + `lib.rs` + `ipc.ts` + `types.ts` 4곳 동기화.
- 답변은 근거 없으면 "자료 없음". 출처(파일명·페이지) 표시.

## 미확정 / 리스크

- 모바일 보안저장 크레이트, Tauri 모바일 빌드/서명, pdf-extract 모바일 컴파일, 클라우드 임베딩 비용, 브라우저 로그인(유예).
