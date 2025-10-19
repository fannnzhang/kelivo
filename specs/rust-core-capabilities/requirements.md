# Requirements: Rust Core Capabilities

## 1. Introduction

To support the long-term health, performance, and maintainability of the Kelivo application, we are initiating a strategic migration of core business logic from Flutter/Dart to a shared Rust backend. This initiative, codenamed "Rust Core Capabilities," aims to build a foundational set of libraries (crates) that will serve as the robust, high-performance backbone for all current and future feature development.

## 2. Business & Technical Goals

- **Performance Improvement:** Offload computationally intensive tasks (data processing, serialization, API communication) to Rust to improve UI responsiveness and overall application speed.
- **Separation of Concerns:** Create a clear boundary between the UI layer (Flutter) and the business logic layer (Rust), making the codebase easier to reason about, test, and maintain.
- **Code Reusability:** Develop a single, shared core logic library that can be used across multiple platforms (iOS, Android, and potentially desktop/web in the future), reducing code duplication.
- **Reliability & Safety:** Leverage Rust's type system and ownership model to eliminate common bugs and create a more stable application core.
- **Developer Velocity:** Provide feature developers with a clear, powerful, and unified set of tools for common tasks like database access and configuration management, accelerating future development.

## 3. Scope

This initiative defines and delivers a comprehensive Rust Core that subsumes the Flutter app’s core capabilities behind a stable FRB boundary. It includes:

- Rust workspace at `rust/` with FRB-facing `api` crate and foundational `core` crate.
- Clear FFI boundary: `api` exposes FRB-compatible functions and types; `core` implements business logic and shared utilities.
- Transitional wrappers to keep existing Dart call sites and generated bindings stable during the migration.

Initial and target core capabilities in `core` (invoked via `api`):

- Core error types and mapping to user-facing strings.
- Filesystem/path utilities and zip/WebDAV helpers.
- Markdown media sanitizer pipeline.
- Document text extraction (PDF/DOCX) and text fallback.
- Cryptography: Google Auth JWT signing (service account) and primitives needed by storage/network features.
- Network/HTTP: Provider-agnostic LLM chat client with streaming SSE support and adapter layer (OpenAI-compatible, Anthropic, Google, OpenRouter as first wave), request signing, retry/backoff, and cancellation.
- Async & Concurrency: A single Tokio runtime, async tasks, and bounded threadpool usage for blocking I/O.
- Database/Storage: Embedded SQLite (bundled) for conversations/messages/events with migrations, plus import/export bridges to current Hive data for transition.
- Cross-cutting logging facade, configuration provider, feature flags, and common data models.

## 4. Baseline & Current Capabilities (Repo Reality)

Aligning to the current code and README, the project already integrates flutter_rust_bridge v2 with a single cdylib crate at `rust/` and generated Dart bindings at `lib/src/rust/` (per `flutter_rust_bridge.yaml`: `rust_input: crate::api`, `dart_output: lib/src/rust`). No Rust workspace/multi-crate layout exists today; this spec introduces it in a backward-compatible way.

Existing Rust modules and FFI contracts (exported via `rust/src/api/…`):
- Backup utilities (`rust/src/api/backup.rs` → `lib/src/rust/api/backup.dart`)
  - `create_backup_zip(entries: Vec<BackupZipEntryInput>) -> Result<Vec<u8>, String>`
  - `extract_backup_zip(bytes: Vec<u8>) -> Result<Vec<BackupZipEntry>, String>`
  - `parse_webdav_propfind(xml: String, base_url: String) -> Result<Vec<WebDavEntry>, String>`
  - Notes: Zip path sanitization performed; WebDAV size is `u64` → Dart `BigInt`.
- Markdown media sanitizer (`rust/src/api/markdown_sanitizer.rs` → `lib/src/rust/api/markdown_sanitizer.dart`)
  - `replace_inline_base64_images(markdown: String) -> Result<String, String>` writes decoded images to a filesystem directory.
  - `inline_local_images_to_base64(markdown: String) -> Result<String, String>` converts local image paths to base64 data URLs.
  - Env: `KELIVO_SANITIZER_IMAGE_DIR` (if set) controls output dir; else falls back to Documents/images or temp.
- Document text extraction (`rust/src/api/document_parser.rs` → `lib/src/rust/api/document_parser.dart`)
  - `extract_text_from_pdf(path: String)` and `extract_text_from_docx(path: String)`; plus `read_text_fallback(path: String)`.
- Google Auth JWT signing (`rust/src/api/google_auth.rs` → `lib/src/rust/api/google_auth.dart`)
  - `create_google_auth_jwt(client_email, private_key_pem, token_uri, scopes)`; no key persistence; unit tests provided.
- Simple demo + init (`rust/src/api/simple.rs`)
  - `greet(name) -> String` and `init_app()` calls `flutter_rust_bridge::setup_default_user_utils()`.

Dart integration patterns already in use:
- Feature flags: `USE_RUST` (global) and `USE_RUST_MARKDOWN_SANITIZER` (scoped) in `lib/config/feature_flags.dart`.
- Mock/Real switching implemented in `lib/main.dart` and `lib/src/rust/mock_api.dart`.
- Integration and benchmarks placed under `integration_test/` and `test/benchmark/`.

Conclusion for baseline: The Rust side is currently compute-focused and filesystem-bound helpers, while HTTP, DB, and orchestration remain in Dart. This spec elevates Rust to own the app’s core (network, crypto, storage, async), with progressive, flag‑gated migration that preserves UX and compatibility.

## 5. Gap Analysis vs. Target Core

Planned target vs actual (code) and decision:
- Multi-crate workspace — Not present today. Decision: Adopt now with minimal churn.
  - Workspace root: `rust/Cargo.toml` (workspace only; no [package]).
  - `api` crate: cdylib/staticlib for FRB; keep crate name `rust_lib_Kelivo` to preserve iOS/Android builders.
  - `core` crate: pure Rust library with modules for current and future logic.
  - FRB config updated to `rust_root: rust/api`; `dart_output` unchanged.
- Config provider — Implement a minimal trait-based provider in `core` with default env-backed implementation; allow runtime override via FRB init hook.
- Local DB client — Move to Rust using SQLite (bundled) with migrations; provide import/export with current Hive boxes for compatibility and rollback.
- External HTTP — Move LLM chat networking and SSE streaming to Rust (`reqwest` + `rustls`), with provider adapters; expose cancellations and timeouts; retain Dart-only mode via feature flag.
- Common FFI types — Keep FRB types in `api` crate; convert to/from `core` internal models; document BigInt/u64 mappings.
- Logging — Provide a thin logging facade in `core` backed by `log` crate; `api` hooks to FRB default utils via `init_app()`.

## 6. Non‑Functional & Cross‑Cutting Requirements

- Error model: All exported Rust functions SHALL return `Result<_, String>` with clear, user-actionable messages. THEN Dart SHALL surface errors with consistent UX copy and avoid leaking internal details.
- Platform constraints: File I/O paths must be sandbox-safe (iOS/macOS). THEN Dart SHALL provide path resolution (already via `SandboxPathResolver.init()`). Rust SHALL not assume writable locations beyond env or documents/temp.
- Async runtime: A single Tokio runtime SHALL be used. Blocking I/O SHALL run on bounded threadpools. All FRB-exposed I/O/HTTP APIs SHALL be async and cancellation‑aware.
- Streaming: SSE/streaming responses SHALL be exposed to Dart as FRB Streams with backpressure; chunk parsing SHALL be robust to partial/batched frames and provider idiosyncrasies.
- Performance: Compute tasks SHOULD be faster or comparable to Dart (benchmarks exist for sanitizer). For new functions, add unit tests and simple perf checks where feasible.
- Security: Private keys and sensitive data SHALL remain in memory only for the duration of the call; no persistence. Inputs SHALL be validated and sanitized (e.g., zip paths). No network secrets hardcoded. TLS SHALL use `rustls` with system roots; proxies SHALL be opt‑in via config/env.
- Build & toolchain: Adopt Rust workspace (`rust/`), with FRB v2 config updated to `rust_root: rust/api`. Preserve crate name `rust_lib_Kelivo` to avoid breaking platform builders.
- DB bundling: SQLite SHALL use the bundled build on iOS/Android; migrations SHALL be idempotent and forward‑only.
- MCP usage: Docs/spec tasks SHOULD use MCP provider `context7` for external references (To Confirm). Env only: `MCP_CONTEXT7_URL`, `MCP_CONTEXT7_TOKEN`.
- Notifications: During major task milestones, send local notifications via `alerter` on macOS as per `specify/Alerter-Usage.md` (To Confirm if unavailable in local env; fallback: proceed without blocking).

## 7. Phased Development Strategy

Phase 0 — Workspace scaffold and guardrails
- Create workspace at `rust/` with members `api`, `core` (no root [package]).
- Set `api` crate name to `rust_lib_Kelivo`; move/alias current `src/api/*` progressively.
- Update FRB config: `rust_root: rust/api`, `rust_input: crate::api`, `dart_output: lib/src/rust`.
- Keep Dart generated paths and `RustLib.init()` contract stable.

Phase 1 — Port existing capabilities into `core`
- Implement modules in `core`: `fs`, `zip_webdav`, `markdown`, `document`, `crypto`, `error`, `config`, `logging`, `types`.
- Refactor `api` functions to delegate to `core`, keeping FRB signatures and types stable; add converters between FRB types and `core` models.

Phase 2 — Async runtime & FRB async wiring
- Introduce Tokio runtime in `api`; convert FRB functions that perform I/O to async `Future`s; ensure cancellation and timeouts are plumbed.

Phase 3 — LLM networking (Mock → Real)
- Provide `core::net` and `core::llm` with a provider‑agnostic request/response domain model and adapters for: OpenAI‑compatible, Anthropic, Google (Vertex/Gemini), OpenRouter. Start with Mock provider behind `USE_RUST_LLM=false` → `true`.
- Implement SSE streaming parsing; expose FRB Stream of `ChatEvent` (delta, tool_call, usage, done, error).

Phase 4 — Database (Mock → Real)
- Introduce `core::db` backed by SQLite (rusqlite/sqlx). Provide migrations for conversations/messages/events and import from existing Hive boxes. Gate via `USE_RUST_DB=false` → `true`.

Phase 5 — End‑to‑End switchovers
- Feature flags and Dart glue to choose Rust LLM/DB at runtime; fallback to Dart paths on error.

Phase 6 — Validation and documentation
- Refresh FRB bindings; add integration tests for streaming chat and DB persistence; update developer docs for flags, env, MCP, and alerter points.

## 8. Acceptance Criteria (Examples, EARS)

- WHEN Dart calls `create_backup_zip` with nested entries THEN system SHALL produce a valid zip and preserve dir structure SO THAT restore yields the same tree.
- WHEN Dart calls `extract_backup_zip` on invalid/empty bytes THEN system SHALL return an `Err(String)` with actionable message SO THAT UI can inform users.
- WHEN Markdown contains inline base64 images THEN Rust SHALL emit files under configured directory (or safe fallback) and return updated Markdown SO THAT downstream renderers use filesystem images.
- WHEN Markdown contains local file URLs on supported platforms THEN Rust SHALL inline to data URLs, otherwise leave untouched SO THAT web platform is unaffected.
- WHEN JWT inputs are invalid (empty email, invalid key) THEN Rust SHALL return clear `Err(String)` SO THAT Dart can show precise guidance.

- WHEN Dart starts a streaming LLM chat with provider X and valid config THEN Rust SHALL emit `ChatEvent` deltas over FRB Stream with correct order and final usage SO THAT UI renders progressively.
- WHEN Dart cancels an in‑flight chat THEN Rust SHALL stop network work within 200ms and emit a final cancellation event SO THAT UI transitions cleanly.
- WHEN `USE_RUST_DB=true` and a message is saved THEN Rust SHALL persist to SQLite and a subsequent load SHALL return the same record SO THAT UX is preserved.

## 9. Risks & Open Questions

- Alerter availability across developer machines and CI — To Confirm; fallback is non-blocking.
- MCP tooling availability during local dev/CI — To Confirm; proceed mock-first and do not hardcode tokens.
- Web platform constraints for filesystem-bound helpers — ensure callers gate usage by platform.
- Workspace refactor risk — mitigated by preserving crate name `rust_lib_Kelivo`, updating FRB `rust_root`, and incremental file moves.

## 10. Alignment & References

- FRB config: `flutter_rust_bridge.yaml` (rust_input: `crate::api`, rust_root: `rust/api`, dart_output: `lib/src/rust`)
- Rust workspace: `rust/Cargo.toml` (workspace), crates: `rust/api`, `rust/core`
- iOS/Android builders: `rust_builder/*` referencing crate name `rust_lib_Kelivo`
- Dart feature flags: `lib/config/feature_flags.dart`
- Related docs/specs: `docs/rust_frb_build.md`, `specs/frb-rust-integration/*`, `specs/google-auth-rust-migration/*`
- Dart network baseline: `lib/core/services/api/chat_api_service.dart` (streaming SSE and provider adapters)
- Dart storage baseline: `lib/core/models/conversation.dart` (Hive models) and related usage
- Alerter usage: `specify/Alerter-Usage.md`
- MCP env: `MCP_CONTEXT7_URL`, `MCP_CONTEXT7_TOKEN`
