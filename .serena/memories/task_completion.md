# Task Completion

- Frontend-affecting change: run `npm run typecheck`, `npm test`, and usually `npm run build`.
- Rust-affecting change: from `src-tauri/`, run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.
- Cross-IPC/contract change: run both frontend and Rust validation because `commands.rs`/`lib.rs`/`ipc.ts`/`types.ts` must stay aligned.
- Security/RAG changes: include focused tests where possible for confidential upload rejection, key validation/storage behavior, embedding dimensions, vector search, evidence threshold, and citation assembly.
- Mobile/platform changes: for iOS simulator use `npm run ios:sim`; for Android ensure `ANDROID_HOME`/`NDK_HOME` are set and use Tauri android commands. Do not commit `src-tauri/gen/`.
- CI mirrors: frontend job runs `npm run typecheck`, `npm test`, `npm run build`; Rust job runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; mobile compile job checks Android/iOS target compilation.
- Before final response, mention any validation commands not run and why. If memories were changed, user can run `serena memories check` from the project root.