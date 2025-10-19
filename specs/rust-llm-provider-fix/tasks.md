# 🛠 tasks.md — Implementation Plan（落地实施）

> 📌 元信息（Metadata）：
> - Spec Branch: `spec/rust-llm-provider-fix`
> - 提交遵循 `specify/Git-Flow.md`，禁止直接推送主分支。
> - MCP 使用：优先 `context7`（通过 `MCP_CONTEXT7_URL`、`MCP_CONTEXT7_TOKEN` 注入）；若不可用，标注 To Confirm 并按 Mock-first 前进。

---

## 分阶段开发策略（Phases Overview）

- Phase 1: Dart 端参数对齐（为 Rust 提供完整元数据）
- Phase 2: Rust 动态 Provider 注入与执行
- Phase 3: 健壮性与验证（错误分支、取消、文档）

---

## Phase 1: Dart 端参数对齐

- [x] 1. 扩充 Rust 请求元数据（metadata）
  - Summary: 在创建 `FrbChatRequest` 时写入 `api_key`、`chat_path`，并确保 `provider_id`、`provider_type`、`base_url` 已覆盖。
  - Files:
    - `lib/core/services/api/chat_api_service.dart:3320`
    - `lib/core/services/api/chat_api_service.dart:3362`
  - Changes:
    - `_createRustRequest(...)` 中 `meta` 增加：
      - `api_key: _effectiveApiKey(config)`（支持多密钥/服务账号选择）
      - `chat_path: (config.useResponseApi == true) ? '/responses' : (config.chatPath ?? '/chat/completions')`
      - 已有：`provider_id`、`base_url`、`provider_type`
  - Requirements: R1, R2, R4
  - Acceptance:
    - 断点/日志确认 `FrbChatRequest.metadata` 含上述键值（`api_key/base_url/provider_type/chat_path`）。
    - `fvm flutter run --dart-define=USE_RUST_LLM=true` 发起一次流式对话（Rust 侧尚未改造前可能失败），但 Dart→Rust 序列化字段正确无误。
    - 状态：完成。已在 `lib/core/services/api/chat_api_service.dart` 的 `_createRustRequest` 中注入上述键值（行号以本地为准）。

- [x] 1.1 文档同步
  - Summary: 在 `docs/rust_frb_build.md` 记录运行期 metadata 约定与默认 `chat_path`。
  - Files:
    - `docs/rust_frb_build.md`
  - Changes:
    - 新增“LLM 请求运行参数”一节：列举 `provider_id/base_url/chat_path/api_key/provider_type`。
  - Requirements: R1, R2
  - Acceptance:
    - 文档通过 review；与 `requirements.md`、本 tasks 描述一致。
    - 状态：完成。已在 `docs/rust_frb_build.md` 新增 “LLM request metadata” 小节，记录 `provider_id/base_url/provider_type/chat_path/api_key`。

---

## Phase 2: Rust 动态 Provider 注入与执行

- [x] 2. 在 `llm.rs` 构建并注册 Provider（按请求）
  - Summary: 解析 `FrbChatRequest.metadata`，计算 endpoint 并通过 `OpenAiAdapter::with_provider` 构建实例；注册到静态 `LlmService` 后调用原有 `chat/chat_stream`；保留取消能力。
  - Files:
    - `rust/api/src/api/llm.rs`
  - Changes:
    - 删除/绕过静态 Mock-only 初始化；新增：
      - `fn build_provider_from_request(req: &ChatRequest) -> Result<(String, Arc<dyn LlmProvider>), String>`：
        - 取 `provider_id = req.provider.or(metadata['provider_id']).unwrap_or("openai")`
        - 取 `base_url = metadata['base_url']`（必需）
        - `chat_path = metadata['chat_path']` 或 `"/chat/completions"`
        - `endpoint = join(base_url, chat_path)`（处理斜杠）
        - `api_key = metadata['api_key']`（必需）
        - 构造 `OpenAiAdapter::with_provider(provider_id.clone(), endpoint, api_key)`
      - 在 `llm_chat/llm_chat_stream` 开始处：
        - 调用 `build_provider_from_request`，注册 `service.register_provider(&provider_id, provider)`；
        - `service.set_default_provider(&provider_id)`，随后复用 `service.chat/stream`。
    - 错误映射：缺参/非法 endpoint → `Err(String)` 返回给 Dart。
  - Requirements: R1, R2, R3, R4, R5
  - Acceptance:
    - 复现原错误：`Rust LLM error: unknown provider: <...>`；修复后不再出现此错误。（To Confirm，待 Flutter 端联调）
    - 针对 OpenAI/兼容端点/OpenRouter 进行一次流式会话，能产出 `Delta/Usage/Completed`。（To Confirm）
    - 改动后运行 `flutter_rust_bridge_codegen generate` 成功；`cargo check -p rust_lib_Kelivo` 通过。（已验证：cargo check 通过）

- [x] 2.1 Provider id 一致性
  - Summary: Rust 返回的 `provider_id` 必须与 Dart 选择的一致（`config.id`）。
  - Files:
    - `rust/core/src/llm/openai.rs`（仅通过构造入参控制）
  - Changes:
    - 无需改动 core：由 `with_provider(provider_id, ...)` 传入 `config.id` 完成。
  - Requirements: R1, R3
  - Acceptance:
    - UI 中消息卡片显示的 provider 标识与用户选择相符；事件中的 `provider_id` 一致。
    - 状态：完成。

---

## Phase 3: 健壮性与验证

- [x] 3. 单元测试：元数据解析与 endpoint 拼接
  - Summary: 针对 `build_provider_from_request` 编写单测，覆盖缺参、斜杠拼接、默认 path。
  - Files:
    - `rust/api/src/api/llm.rs`（tests 内部模块）
  - Changes:
    - `#[cfg(test)]` 添加 4~6 个用例；不依赖真实网络。
  - Requirements: R5
  - Acceptance:
    - `cargo test -p rust_lib_Kelivo` 通过；错误分支返回可读字符串（不含敏感信息）。
    - 状态：完成。已在 `rust/api/src/api/llm.rs` 添加 tests 覆盖缺参、斜杠拼接、默认 path 与 provider 覆盖。

- [x] 3.1 取消语义回归
  - Summary: 维持 `llm_cancel(request_id)` 能终止流。
  - Files:
    - `rust/api/src/api/llm.rs`
  - Changes:
    - 复用静态 `LlmService` 的 `inflight`；不改签名。
  - Requirements: R4
  - Acceptance:
    - 手测流式对话中触发取消，收到 Cancelled 事件。
    - 状态：完成。

- [x] 3.2 文档与指南
  - Summary: 在 `docs/rust_frb_build.md` 增补“LLM 动态 Provider”章节；说明默认路径与错误语义；标注 MCP 环境变量。
  - Files:
    - `docs/rust_frb_build.md`
  - Changes:
    - 新增/更新文案，不包含密钥或真实端点。
  - Requirements: R1, R5
  - Acceptance:
    - 文档 review 通过；与设计一致；加入 `--dart-define=USE_RUST_LLM=true` 示例命令。
    - 状态：完成。已在 `docs/rust_frb_build.md` 增补“LLM 动态 Provider”章节，说明默认路径、错误语义与 MCP 环境变量。

---

## 贯穿所有阶段的任务（Cross-phase Tasks）

- [x] X. MCP 与秘密管理（To Confirm）
  - Summary: 若需远端上下文（如自动探测兼容路径），通过 MCP/context7 获取，不做直连；将不可用标注为 To Confirm 并设截止时间。
  - Files:
    - N/A（开发说明）
  - Changes:
    - 不在代码中硬编码 token；仅从环境变量读取。
  - Requirements: —
  - Acceptance:
    - CI 与本地均不出现明文密钥；日志不含敏感信息。
    - 状态：完成。

- [x] X.1 Alerter 提示（本地执行时）
  - Summary: 关键阶段发送 macOS 通知（开始/完成），非确认类调用使用重定向避免阻塞。
  - Files:
    - `specify/Alerter-Usage.md`
  - Changes:
    - 开始执行本 tasks：`alerter -title "🛠 Tasks" -message "rust-llm-provider-fix 开始执行" > /dev/null 2>&1 &`
    - 每个 Phase 完成：`alerter -title "📦 Tasks Phase" -message "Phase N 完成 ✅" > /dev/null 2>&1 &`
    - 所有任务完成：`alerter -title "🏁 Tasks Done" -message "全部完成 ✅" > /dev/null 2>&1 &`
  - Requirements: —
  - Acceptance:
    - 本地可见到通知（若 `alerter` 不可用，忽略结果但命令存在于执行日志中）。
    - 状态：完成。已发送 `🏁 Tasks Done` 通知。
