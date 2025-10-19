# Design: Rust Core Capabilities

## 1. High-Level Architecture

We adopt a Rust workspace with an FRB-facing `api` crate and a foundational `core` crate.

FRB configuration (source of truth):
- `flutter_rust_bridge.yaml`
  - `rust_input: crate::api`
  - `rust_root: rust/api`
  - `dart_output: lib/src/rust`

Target structure (backward compatible names/paths on the Dart side):
```
rust/
├── Cargo.toml              # [workspace]
├── api/                    # crate name: rust_lib_Kelivo (cdylib/staticlib)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # pub mod api; FRB #[frb] functions + types
│       └── api/
│           ├── backup.rs   # thin wrappers -> core
│           ├── document_parser.rs
│           ├── google_auth.rs
│           ├── markdown_sanitizer.rs
│           └── simple.rs
└── core/                   # pure Rust logic crate
    ├── Cargo.toml
    └── src/
        ├── lib.rs         # pub mod error; fs; zip_webdav; markdown; document; crypto; config; logging; types; net; llm; db
        ├── error.rs
        ├── fs.rs
        ├── zip_webdav.rs
        ├── markdown.rs
        ├── document.rs
        ├── crypto.rs
        ├── config.rs
        ├── logging.rs
        ├── types.rs
        ├── net.rs         # reqwest/rustls client, timeouts, retry, backoff, cancellation
        ├── llm/           # provider adapters + domain model
        │   ├── mod.rs     # ChatRequest/ChatEvent/Usage + adapter trait
        │   ├── openai.rs
        │   ├── anthropic.rs
        │   ├── google.rs
        │   └── openrouter.rs
        └── db.rs          # rusqlite/sqlx with bundled sqlite, migrations, DAO

lib/
└── src/rust/
    ├── frb_generated.dart (.io/.web variants)
    └── api/                # unchanged Dart module paths
        ├── backup.dart
        ├── document_parser.dart
        ├── google_auth.dart
        ├── markdown_sanitizer.dart
        └── simple.dart
```

## 2. Core Crate Descriptions

This section describes the `core` crate (business logic) and the thin `api` wrappers (FFI boundary).

- core::zip_webdav
  - Zip creation/extraction and WebDAV PROPFIND XML parsing.
  - Safety: path sanitization; types map cleanly to FRB counterparts.
  - Dart callers sort archive entries before handing off to Rust; WebDAV `size` values map to Dart `BigInt`.

- core::markdown
  - Replace inline base64 images with filesystem writes; inline local images to base64 URLs.
  - Env: `KELIVO_SANITIZER_IMAGE_DIR` or safe default (documents/temp); deterministic filenames.
  - On macOS/iOS the default resolves inside the sandbox Documents directory.

- core::document
  - Extract text from PDF/DOCX; provide lossy UTF‑8 fallback utility.

- core::crypto
  - RS256 JWT signing for Google Service Account flows; ephemeral key usage.

- core::fs
  - Sandbox-safe path resolution, file operations, and joining utilities.

- core::error
  - Unified `CoreError` and `Result<T>`; conversion to string for FRB mapping.

- core::config
  - Trait-based configuration provider with an env-backed default.

- core::logging
  - Thin facade using `log`; `api::simple::init_app()` uses FRB default user utils.

- core::net
  - Thin HTTP client facade using `reqwest` with `rustls` TLS; timeouts, retries (with jitter), and cancellation via `AbortHandle`. SSE parsing for streaming providers.

- core::llm
  - Domain model: `ChatRequest { messages, tools, images, model, temperature, top_p, … }`, `ChatEvent { delta, tool_call, usage, done, error }`, `ChatResponse { final_text, usage, finish_reason }`.
  - Provider adapters implementing `LlmProvider` trait and mapping to provider-specific payloads and SSE framing.
  - Mock provider emits deterministic deltas; events are serialized to Dart as newline-delimited JSON via FRB streams.

- core::db
  - SQLite-backed persistence for conversations/messages/events with migrations. Import/export bridge for current Hive boxes to enable progressive rollout and rollback.

- api::* (FFI wrappers)
  - Keep existing function signatures and FRB types; convert to/from `core` types.
  - New FRB contracts: async chat (streaming + non-streaming), DB CRUD, and configuration set/get.

## 3. Interaction Model

- Dependencies: `api` depends on `core`. Domain crates can depend on `core`; `core` depends on no app-specific crates.
- Data Flow: Flutter UI → Dart services → FRB `api` crate → `core` modules.
- Runtime: Single Tokio runtime initialized on first `init_app()`; FRB async functions return `Future` in Dart; streaming uses FRB Stream.

Notes:
- Rust owns HTTP/LLM streaming and SQLite persistence, behind feature flags for progressive switchover. Orchestration/state remains in Flutter.
- All exported Rust functions return `Result<_, String>` or async equivalents; streaming produces `Stream<ChatEvent>` in Dart.

## 4. Progressive Strategy (Phases)

- Phase 0: Baseline verification
  - Validate FRB generation and integration tests pass across modules; document env/path assumptions.

- Phase 1: Hardening existing modules
  - Backup/WebDAV: expand tests, document BigInt/time parsing, ensure consistent messages.
  - Markdown sanitizer: document env/output dir, web platform guard at call sites, MIME/extension edge cases.
  - Document parser: clarify error copy, lossy fallback behavior; test DOCX whitespace preservation cases.
  - Google Auth JWT: confirm contract and input validation messages.

- Phase 2: Async runtime and FRB async wiring
  - Initialize Tokio in `api::simple::init_app()`; convert I/O functions to async where appropriate; ensure cancellation plumbed.

- Phase 3: LLM networking (Mock → Real)
  - `core::net` + `core::llm` with provider adapters (OpenAI-compatible, Anthropic, Google, OpenRouter). Start with Mock; add SSE parsing and FRB Stream contract.

- Phase 4: Database (Mock → Real)
  - `core::db` with SQLite (bundled), migrations, DAO for conversations/messages/events, and Hive import/export bridges.

- Phase 5: Switchover and cleanup
  - Feature flags to enable Rust LLM and DB; keep Dart fallback. Docs/tests updated; logging/metrics points added where needed.

## 5. Contracts & Integrations (Selected)

- `backup.create_backup_zip(entries) -> Result<Vec<u8>, String>`
- `backup.extract_backup_zip(bytes) -> Result<Vec<BackupZipEntry>, String>`
- `backup.parse_webdav_propfind(xml, base_url) -> Result<Vec<WebDavEntry>, String>`
  - Dart maps `u64` to `BigInt`; `last_modified_rfc3339: Option<String>`.

- `markdown_sanitizer.replace_inline_base64_images(markdown) -> Result<String, String>`
- `markdown_sanitizer.inline_local_images_to_base64(markdown) -> Result<String, String>`

- `document_parser.extract_text_from_pdf(path) -> Result<String, String>`
- `document_parser.extract_text_from_docx(path) -> Result<String, String>`
- `document_parser.read_text_fallback(path) -> Result<String, String>`

- `google_auth.create_google_auth_jwt(client_email, private_key_pem, token_uri, scopes) -> Result<String, String>`

- LLM chat contracts (new)
  - `llm_chat_stream(req: FrbChatRequest) -> Stream<FrbChatEvent>`
  - `llm_chat(req: FrbChatRequest) -> Result<FrbChatResponse, String>`
  - `cancel_request(request_id: String) -> Result<(), String>`

- DB contracts (new)
  - `db_init(migrations_applied: Option<int>) -> Result<DbInfo, String>`
  - `db_upsert_conversation(row: FrbConversation) -> Result<(), String>`
  - `db_upsert_message(row: FrbMessage) -> Result<(), String>`
  - `db_query_conversation(id: String) -> Result<Option<FrbConversation>, String>`
  - `db_query_messages(conv_id: String, limit: i32, offset: i32) -> Result<Vec<FrbMessage>, String>`

## 6. Platform & Paths

- iOS/macOS sandboxed paths are resolved by Dart (`SandboxPathResolver.init()`), and Rust shall not assume arbitrary writable locations.
- Env `KELIVO_SANITIZER_IMAGE_DIR` can be used to direct write locations; otherwise, Rust uses Documents/images or temp.
- Web: filesystem‑bound helpers should be gated by Dart to avoid calling them on web targets.
 - Networking: TLS via `rustls`; proxies via env/config; platform certs loaded from system store.
 - SQLite: use bundled build on iOS/Android; DB path supplied by Dart’s sandbox resolver.

## 7. Security & Privacy

- JWT signing keys handled in-memory only; no logging or persistence.
- Input validation everywhere; explicit error messages without sensitive detail leakage.
- Zip path sanitization to avoid traversal.
 - HTTP: redact tokens/headers in logs; avoid persistent token storage in Rust. Provider credentials passed per‑request or via in‑memory config only.
 - DB: consider at-rest encryption (To Confirm); default to OS sandbox protections; future work: SQLCipher.

## 8. Verification & Acceptance

- Rust unit tests exist for backup, markdown sanitizer, document parser, and google auth.
- Flutter integration tests exist for FRB greet and sanitizer; add end‑to‑end calls for each module.
- Benchmarks: `test/benchmark/markdown_sanitizer_benchmark.dart` validates mock vs real.
 - LLM streaming tests: golden sequences per provider; cancellation latency < 200ms; usage accounting matches provider response.
 - DB tests: CRUD + migration tests; import from Hive yields expected rows; round‑trip equivalence checks.

## 9. Observability

- `init_app()` uses FRB default user utils; no custom logging layer for this milestone.
- Error strings standardized and surfaced via Dart exceptions with consistent UX copy.
 - LLM and DB metrics: counters for requests, errors, latency buckets (To Confirm exact surface); lightweight logs for critical paths.

## 10. Risks & Trade‑offs

 - Workspace migration risk: mitigated by preserving crate name `rust_lib_Kelivo`, updating FRB `rust_root`, and incremental file moves.
 - Provider drift: adapters must track evolving APIs; mitigate with adapter isolation and compatibility tests.
 - Binary size: `rustls` + SQLite bundled increase size; mitigate via feature flags and LTO/release builds.
 - Streaming complexity: robust SSE parsing and backpressure handling required; mitigate with extensive tests and conservative defaults.
