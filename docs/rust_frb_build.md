# FRB Build & Switch Guide

## Workspace layout

Rust logic is organised as a workspace under `rust/`:

| Crate | Purpose |
| --- | --- |
| `rust/api` (`rust_lib_Kelivo`) | FRB-facing crate exporting thin wrappers. |
| `rust/core` (`kelivo_core`) | Pure Rust business logic (backup, markdown, document parsing, crypto, LLM, etc.). |

`flutter_rust_bridge.yaml` points `rust_root` to `rust/api` and `rust_input` to `crate::api` so that codegen only touches the FFI crate.

## Build commands

1. Refresh Flutter deps: `fvm flutter pub get`
2. Re-generate FRB bindings (Dart + Rust):
   ```bash
   flutter_rust_bridge_codegen generate
   ```
3. Quality gates:
   - `fvm dart format lib test --set-exit-if-changed`
   - `fvm dart analyze`
   - `fvm flutter test`

## Runtime switches

- Mock mode: `fvm flutter run --dart-define=USE_RUST=false`
- Real mode: `fvm flutter run --dart-define=USE_RUST=true`

Feature flags (see `lib/config/feature_flags.dart`):

| Flag | Description |
| --- | --- |
| `USE_RUST_LLM` | Route chat generation through Rust LLM service. |
| `USE_RUST_DB` | Enable SQLite-backed persistence via Rust core. |

## Environment variables

- `KELIVO_SANITIZER_IMAGE_DIR`: Optional absolute path for the markdown media sanitizer to persist extracted images. If unset, the sanitizer writes to `${Documents}/images` (on iOS/macOS this resolves inside the app sandbox). Ensure the directory is writable; the sanitizer will create it when missing.
- `MCP_CONTEXT7_URL`, `MCP_CONTEXT7_TOKEN`: MCP/context7 endpoint used for tooling notes—configure via shell and never commit secrets.

## LLM mock & cancellation

The Rust LLM service (`kelivo_core::llm`) ships with a mock provider by default. Streaming APIs surface newline-delimited JSON events over FRB (`lib/src/rust/api/llm.dart`). Cancellation is exposed via `llm_cancel` and coordinated through `tokio_util::sync::CancellationToken`.

## LLM request metadata

When `USE_RUST_LLM` is enabled, Dart builds `FrbChatRequest.metadata` with runtime parameters so Rust can route to the correct endpoint without hardcoding:

- provider_id: Provider identifier from `ProviderConfig.id`.
- base_url: Effective base URL, if provided.
- provider_type: One of `openai`, `google`, `claude` (when explicitly set).
- chat_path: `"/responses"` if `useResponseApi == true`, else `chatPath ?? "/chat/completions"`.
- api_key: Effective API key, honoring multi-key selection.

Secrets must come from environment or the app’s secure storage and must not be committed. For CI or tooling, configure MCP/context7 via `MCP_CONTEXT7_URL` and `MCP_CONTEXT7_TOKEN`.

## LLM 动态 Provider

When `USE_RUST_LLM=true`, Rust dynamically constructs the LLM provider per request using the metadata above, without hardcoded endpoints:

- Endpoint join: `endpoint = join(trim_end(base_url, '/'), trim_start(chat_path, '/'))`.
- Default path: if `chat_path` is missing, uses `/chat/completions`.
- Provider ID: `request.provider` overrides `metadata.provider_id`; falls back to `openai`.
- Required keys: missing `base_url` or `api_key` returns readable errors
  - `missing required metadata: base_url`
  - `missing required metadata: api_key`

Cancellation is preserved via `llm_cancel(request_id)`. During a streaming session, cancelling emits a `Cancelled` event before teardown.

Example run:

```
fvm flutter run --dart-define=USE_RUST=true --dart-define=USE_RUST_LLM=true
```

MCP/context7
- Prefer retrieving any external context through MCP. Configure via `MCP_CONTEXT7_URL` and `MCP_CONTEXT7_TOKEN` only. If unavailable, proceed Mock-first and mark as “To Confirm”.

## Notes

- FRB runtime: `flutter_rust_bridge` v2.
- `mock` provider is registered during library init; real adapters (OpenAI, Anthropic, Google, OpenRouter) live in `rust/core/src/llm/`.
- Always re-run codegen after adding FRB annotations (`#[frb]`).
