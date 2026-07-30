# Conventions

- Language: comments and UI strings are Korean; code identifiers are English.
- Security boundary: frontend checks are convenience only. Rust IPC, especially `upload_document`, is the final guard for rejecting `Sensitivity::Confidential`.
- New/changed IPC must keep four surfaces synchronized: Rust `commands.rs`, Tauri invoke handler in `lib.rs`, frontend wrapper `src/lib/ipc.ts`, and frontend shared types in `src/types.ts`.
- Serialization contract: Rust uses `serde(rename_all = "camelCase")` for IPC structs and `lowercase` for provider/sensitivity enums to match TS types.
- No local LLM paths in mobile: do not introduce Ollama, local embeddings, browser-login hacks, or desktop two-collection security branching.
- API keys: never store in TS/localStorage/files. Use `security::{set_api_key,get_api_key,has_api_key,delete_api_key}` through IPC; secure-store commands run through `spawn_blocking`/`off_main` to avoid Android deadlocks.
- RAG behavior: answers must be grounded in uploaded docs; if retrieval score is below threshold, return fixed `NO_EVIDENCE` without generation. Citations should include filename and page.
- Retrieval: hybrid search combines cosine and lexical terms; keep this for Korean visa-specific exact terms and codes.
- Locks/async: do not hold `DocStore` or `VectorStore` mutex guards across `.await`; current `commands.rs` deliberately releases locks before network calls.
- Frontend state: provider/model are localStorage settings; chat sessions are localStorage; API key presence is checked via IPC and key values are never read back to UI.
- Mobile UI: bottom 3-tab shell; tab panes remain mounted and toggle `hidden` to preserve state. File upload reads bytes in frontend because Android returns `content://` URIs.
- Harness workflow: specs in `docs/specs/`, ADRs in `docs/adr/`, tasks in `todos/active`/`todos/done`; `todos/BACKLOG.md` is source of pending work.
- Commit convention: `type :: 메시지` with Conventional Commit-like type, e.g. `feat :: 문서 업로드 구현`.