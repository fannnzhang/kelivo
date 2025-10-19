# Tasks: Rust Core Capabilities

> 📌 元信息（Metadata）：
> - Spec Branch: `specs/rust-core-capabilities`
> - 所有提交必须推送到该分支，不得直接提交到主分支。
> - 每个 Phase 执行完成的验收条件之一：对应修改已按 `specify/Git-Flow.md` 规范提交。

---

This document outlines the master plan for establishing the foundational Rust libraries for the Kelivo application. Each major task will involve the creation of its own detailed spec documents before implementation begins.

## Phase 0: Workspace and Foundation

- [x] 0. Workspace scaffold (api + core)
  - Summary: Create `rust/` workspace with `api` (FFI) and `core` (logic) crates.
  - Files:
    - `rust/Cargo.toml`, `rust/api/Cargo.toml`, `rust/core/Cargo.toml`
    - `rust/api/src/lib.rs`, `rust/api/src/api/*`, `rust/core/src/*`
  - Changes:
    - Add `[workspace]` at `rust/Cargo.toml` with members `api`, `core`.
    - Set `rust/api` crate name to `rust_lib_Kelivo` with `cdylib`, `staticlib`.
    - Scaffold `core` modules: `error`, `fs`, `zip_webdav`, `markdown`, `document`, `crypto`, `config`, `logging`, `types`.
  - Requirements: (Alignment, NFR)
  - Acceptance:
    - `cargo check -p rust_lib_Kelivo` and `cargo check -p core` succeed locally.
    - 提交本次修改的代码（遵循 `specify/Git-Flow.md`）。

- [x] 0.1 FRB config switch to workspace
  - Summary: Point FRB to `rust/api` crate while keeping Dart output unchanged.
  - Files:
    - `flutter_rust_bridge.yaml`
  - Changes:
    - Update to `rust_root: rust/api`, keep `rust_input: crate::api`, `dart_output: lib/src/rust`.
  - Requirements: (FRB v2)
  - Acceptance:
    - `flutter_rust_bridge_codegen generate` succeeds; generated Dart path unchanged.
    - 提交本次修改的代码（遵循 `specify/Git-Flow.md`）。

- [x] 0.2 rust_builder alignment
  - Summary: Ensure iOS/Android/macOS/Windows builders target the `rust_lib_Kelivo` crate within the workspace.
  - Files:
    - `rust_builder/*` (CMakeLists.txt, podspecs, gradle)
  - Changes:
    - Verify paths still point to `../../rust` and crate name `rust_lib_Kelivo`; no rename required.
    - If needed, pass `-p rust_lib_Kelivo` in builder flags (To Confirm via MCP/context7 notes).
  - Requirements: (Build)
  - Acceptance:
    - Local platform build succeeds or remains functionally equivalent to baseline.
    - 提交本次修改的代码（遵循 `specify/Git-Flow.md`）。

- [x] 0.3 Migrate modules with thin wrappers
  - Summary: Move implementations to `core` and keep `api` FRB signatures stable.
  - Files:
    - From `rust/src/api/*` to `rust/core/src/*` and `rust/api/src/api/*`
  - Changes:
    - Implement logic in `core`; expose FRB in `api` with conversions between FRB types and `core` models.
    - Preserve names/signatures for Dart code compatibility.
  - Requirements: (Compatibility)
  - Acceptance:
    - All existing FRB-exported functions compile and behave as before.
    - 提交本次修改的代码（遵循 `specify/Git-Flow.md`）。

- [x] 0.4 Document env and platform path behavior
  - Summary: Clarify `KELIVO_SANITIZER_IMAGE_DIR` and iOS/macOS sandbox behavior.
  - Files:
    - `docs/rust_frb_build.md`, `specs/rust-core-capabilities/design.md`
  - Changes:
    - Add notes for env var and default directories; remind gating on web.
  - Requirements: (NFR: platform constraints)
  - Acceptance:
    - Docs updated; reviewed against code.
    - 提交本次修改的代码（遵循 `specify/Git-Flow.md`）。

## Phase 1: Core Service Implementation

This phase can be parallelized where dependencies allow.

- [x] 1. Backup zip + WebDAV hardening
  - Summary: Strengthen path normalization contract and WebDAV parsing edge cases.
  - Files:
    - `rust/core/src/zip_webdav.rs`, `rust/api/src/api/backup.rs`, `lib/core/services/backup/data_sync.dart`
  - Changes:
    - Verify/extend tests for zip traversal prevention; document BigInt and RFC3339 mapping.
    - Confirm Dart caller sorts/sanitizes archive paths before zipping.
  - Requirements: (Req, NFR: error model)
  - Acceptance:
    - Rust tests pass for traversal/empty zip/propfind cases; Dart e2e lists uploads.
    - 提交本次修改的代码（遵循 `specify/Git-Flow.md`）。

- [x] 1.1 Markdown sanitizer clarity
  - Summary: Document env/output dir and expand MIME/extension coverage.
  - Files:
    - `rust/core/src/markdown.rs`, `rust/api/src/api/markdown_sanitizer.rs`, `docs/rust_frb_build.md`
  - Changes:
    - Add/verify tests for additional MIME types; ensure stable filenames; doc env var behavior.
  - Requirements: (Req: platform constraints)
  - Acceptance:
    - Tests pass; doc updated; benchmark still favorable or comparable.
    - 提交本次修改的代码（遵循 `specify/Git-Flow.md`）。

- [x] 1.2 Document parser messages
  - Summary: Clarify error copy for file open/ZIP read/XML parse/UTF-8 decode.
  - Files:
    - `rust/core/src/document.rs`, `rust/api/src/api/document_parser.rs`
  - Changes:
    - Review/normalize error strings; add tests for whitespace preservation and lossy fallback.
  - Requirements: (NFR: error model)
  - Acceptance:
    - Tests pass; messages consistent and actionable.
    - 提交本次修改的代码（遵循 `specify/Git-Flow.md`）。

- [x] 1.3 Google Auth JWT contract
  - Summary: Validate inputs and confirm contract; integration test on Dart side.
  - Files:
    - `rust/core/src/crypto.rs`, `rust/api/src/api/google_auth.rs`, `lib/core/services/api/google_service_account_auth.dart`
  - Changes:
    - Ensure clear errors for empty email/token_uri/invalid key/empty scopes; doc security posture.
  - Requirements: (Security, Req)
  - Acceptance:
    - Rust unit tests pass; Dart flow obtains token and proceeds; errors mapped cleanly.
    - 提交本次修改的代码（遵循 `specify/Git-Flow.md`）。

## Phase 2: Validation and Documentation

- [ ] 2. End-to-End Integration Test
  - Summary: 为各 Rust 能力提供端到端调用验证，覆盖关键路径。
  - Files:
    - `integration_test/rust_greet_test.dart`
    - `integration_test/simple_test.dart`
  - Changes:
    - 为每个 Rust 模块至少新增或扩展 1 个端到端用例，确保初始化、调用、错误映射完整。
  - Requirements: (NFR: 集成稳定性)
  - Acceptance:
    - 本阶段相关测试在 CI 与本地均通过（`fvm flutter test`）。
    - 提交本次修改的代码（遵循 `specify/Git-Flow.md`）。

- [ ] 2.1 Developer Documentation
  - Summary: 更新开发文档，补充 FRB 生成步骤、Feature Flag、能力目录与 MCP/context7 使用说明。
  - Files:
    - `docs/rust_frb_build.md`
    - `specs/rust-core-capabilities/design.md`
  - Changes:
    - 补充 FRB 代码生成命令与注意事项；列出 Feature Flag 与能力清单；标注 `MCP_CONTEXT7_URL`、`MCP_CONTEXT7_TOKEN` 的使用与不可提交约束；说明工作区布局与 crate 名称约束（`rust_lib_Kelivo`）。
  - Requirements: (Process, Security)
  - Acceptance:
    - 文档完成度通过人工审阅，与代码与脚本一致；敏感信息未入库。
    - 提交本次修改的代码（遵循 `specify/Git-Flow.md`）。

---

## Phase 3: LLM Networking (Mock → Real)

- [x] 3. Provider-agnostic domain + FRB types
  - Summary: Define `FrbChatRequest`, `FrbChatEvent`, `FrbChatResponse` in `rust/api/src/api/llm_types.rs` and map to `core::llm` domain.
  - Files:
    - `rust/core/src/llm/mod.rs`, `rust/api/src/api/llm_types.rs`, `lib/src/rust/api/llm.dart`
  - Changes:
    - Create domain structs/enums for requests/events/responses; add conversions to FRB types.
  - Requirements: (Req: streaming, async)
  - Acceptance:
    - `cargo check` passes; FRB codegen succeeds; Dart types available for integration.
    - 提交本次修改的代码。

- [x] 3.1 Mock provider and streaming contract
  - Summary: Implement a mock `LlmProvider` that streams deterministic deltas; expose `llm_chat_stream` and `llm_chat`.
  - Files:
    - `rust/core/src/llm/mock.rs`, `rust/api/src/api/llm.rs`
  - Changes:
    - Provide FRB stream via async generator; support cancellation by `request_id`.
  - Requirements: (Mock-first)
  - Acceptance:
    - Integration test in Dart consumes stream and renders deltas in order; cancel test completes within 200ms.
    - 提交本次修改的代码。

- [x] 3.2 Net client and OpenAI-compatible adapter
  - Summary: Add `core::net` (reqwest/rustls) and `core::llm::openai` adapter supporting non-stream + SSE.
  - Files:
    - `rust/core/src/net.rs`, `rust/core/src/llm/openai.rs`, `rust/api/src/api/llm.rs`
  - Changes:
    - Implement timeouts, retry/backoff, and SSE chunk parsing with robust framing; redact tokens in logs.
  - Requirements: (Security, Performance)
  - Acceptance:
    - E2E against a mockable OpenAI-compatible endpoint passes; golden event sequence stable.
    - 提交本次修改的代码。

- [x] 3.3 Provider adapters: Anthropic, Google, OpenRouter
  - Summary: Implement adapters incrementally; normalize tool-calls, usage accounting, and finish reasons.
  - Files:
    - `rust/core/src/llm/anthropic.rs`, `rust/core/src/llm/google.rs`, `rust/core/src/llm/openrouter.rs`
  - Changes:
    - Map provider-specific payloads to domain; handle streaming variants and multi-round flows.
  - Requirements: (Compatibility)
  - Acceptance:
    - Provider-specific tests pass; integration toggles select adapters correctly via Dart config.
    - 提交本次修改的代码。

---

## Phase 4: Database (Mock → Real)

- [x] 4. Schema and migrations
  - Summary: Introduce SQLite with bundled build; define schema for conversations/messages/events and migrations.
  - Files:
    - `rust/core/src/db.rs`, `rust/api/src/api/db.rs`
  - Changes:
    - Create migration runner; idempotent, forward-only; DB path provided by Dart via FRB.
  - Requirements: (Persistence)
  - Acceptance:
    - Unit tests for migration up/down scenarios; open/close DB without leaks.
    - 提交本次修改的代码。

- [x] 4.1 CRUD + import/export bridges
  - Summary: Implement CRUD APIs and Hive import/export for progressive rollout.
  - Files:
    - `rust/core/src/db.rs`, `rust/api/src/api/db.rs`, `lib/core/models/conversation.dart`
  - Changes:
    - Map Hive models to DB rows; create import path and verify round-trip equivalence.
  - Requirements: (Compatibility)
  - Acceptance:
    - E2E import preserves message history; queries return expected slices.
    - 提交本次修改的代码。

---

## Phase 5: End-to-End Switchovers

- [x] 5. Feature flags and Dart glue
  - Summary: Add `USE_RUST_LLM` and `USE_RUST_DB` flags and wire callsites to Rust.
  - Files:
    - `lib/config/feature_flags.dart`, `lib/core/services/api/chat_api_service.dart`, `lib/core/providers/*`
  - Changes:
    - Route chat/network/storage through Rust behind flags; fallback to Dart on error.
  - Requirements: (Mock→Real, UX preservation)
  - Acceptance:
    - App runs in both modes; toggling flags switches implementations without regressions.
    - 提交本次修改的代码。

---

## Phase 6: Validation and Documentation (LLM + DB)

- [ ] 6. Streaming + cancellation tests
  - Summary: Golden streaming sequences per provider and cancellation latency checks.
  - Files:
    - `integration_test/*`
  - Changes:
    - Add E2E tests that verify order, content, usage accounting, and cancellation.
  - Requirements: (Reliability)
  - Acceptance:
    - Tests pass locally and in CI; flakiness budget respected.
    - 提交本次修改的代码。

- [ ] 6.1 Developer documentation updates
  - Summary: Update `docs/rust_frb_build.md` with runtime flags, env, MCP notes, and alerter checkpoints.
  - Files:
    - `docs/rust_frb_build.md`, `specs/rust-core-capabilities/design.md`
  - Changes:
    - Document FRB async/streaming usage, SQLite bundling, and provider adapter configuration.
  - Requirements: (Docs)
  - Acceptance:
    - Docs reviewed; consistent with code; no secrets committed.
    - 提交本次修改的代码。

## 贯穿所有阶段的任务（Cross-phase Tasks）

- [ ] X. Alerter notifications for progress
  - Summary: Send macOS notifications at phase boundaries (start/complete) via `alerter`.
  - Files:
    - `specify/Alerter-Usage.md`
  - Changes:
    - Add simple commands to CI/dev scripts or manual steps; if `alerter` is unavailable, mark To Confirm and proceed without blocking.
  - Requirements: (Process)
  - Acceptance:
    - Notifications appear on compatible macOS dev setups; non-blocking on others.
    - 提交本次修改的代码（遵循 `specify/Git-Flow.md`）。

- [ ] X. MCP usage for external references (To Confirm)
  - Summary: Use `context7` MCP for FRB/crate docs lookup during spec updates; avoid direct network calls.
  - Files:
    - `specs/*`, environment
  - Changes:
    - Reference tools/capabilities used; rely on `MCP_CONTEXT7_URL`, `MCP_CONTEXT7_TOKEN` only.
  - Requirements: (Process, Security)
  - Acceptance:
    - Specs reference MCP tools; secrets not committed.
    - 提交本次修改的代码（遵循 `specify/Git-Flow.md`）。
