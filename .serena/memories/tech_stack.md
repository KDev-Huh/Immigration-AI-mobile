# Tech Stack

- Package manager: npm with `package-lock.json`; Node 22 in CI.
- Frontend: React 18 + TypeScript 5.5 + Vite 5.4; path alias `@/* -> src/*`; strict TS with `noUnusedLocals`/`noUnusedParameters`; Vitest 2 + jsdom for frontend tests.
- Tauri: v2 mobile/desktop shell; plugins `@tauri-apps/plugin-dialog` and `@tauri-apps/plugin-fs`; dev server fixed to port 1420 with `strictPort` and mobile HMR port 1421 when `TAURI_DEV_HOST` is set.
- Rust: edition 2021, rust-version 1.77; Tauri lib crate `immigration_ai_mobile_lib` builds `staticlib`, `cdylib`, and `rlib` for mobile.
- Rust deps: `tauri`, dialog/fs plugins, `serde`, `serde_json`, `tokio`, `anyhow`, `uuid`, `sha2`, `pdf-extract`, `reqwest` with rustls/webpki roots.
- Secure storage deps: desktop uses `keyring`; iOS uses `security-framework`; Android uses `jni`, `base64`, and explicit `wry` access for JavaVM/Activity dispatch.
- LLM/embedding: OpenAI chat default in Rust is `gpt-4o`; Anthropic default is `claude-sonnet-5`; Gemini default is `gemini-3.6-flash`; OpenAI embedding model is `text-embedding-3-small` with 1536 dimensions. Non-OpenAI chat providers still require OpenAI for embeddings.
- Vector DB: local JSON collection under app data `vectors`; single collection only, no all/leakable split.
- CI targets: Ubuntu for frontend/Rust checks; macOS for mobile target compile checks; Android NDK r25c; iOS minimum 14.0, Android minSdk 24.
- Tauri config: product `비자 행정사 AI`, identifier `com.immigrationai.mobile`; CSP allows OpenAI and Anthropic APIs only for external network calls.