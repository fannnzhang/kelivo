# 🧠 design.md — Rust LLM Provider 下沉修复方案

## 0. 元信息（Meta）

- Feature / Bug 名称：`rust-llm-provider-fix`
- Spec 路径：`specs/rust-llm-provider-fix/design.md`
- 版本 / 日期：v1.0 · 2025-10-19
- 关联：`requirements.md`、`tasks.md`、`docs/rust_frb_build.md`
- 所有者 / 评审人：Kelivo Core / @maintainers
- 范围声明（Scope / Non-goals）
  - In Scope：
    - 按请求动态实例化 LLM Provider（OpenAI 兼容），保持 FRB 接口不变。
    - 取消静态 Mock-only 注册，解决 “unknown provider” 报错。
    - 维持现有取消机制（request_id 维度）与流式事件协议。
  - Out of Scope：
    - 非 OpenAI 兼容（原生 Gemini/Claude 专用协议）的直连实现。
    - 新增 UI 交互与模型管理改造（Flutter 端保持既有模式）。

---

## 1. 项目基线与约束（Baseline & Constraints）

- 架构与平台：Flutter + FRB（`rust/api` FFI 包装，`rust/core` 业务）；当前 `llm.rs` 以静态 `LlmService(mock)` 提供服务。
- 模块边界与现有约定：
  - Dart 通过 `FrbChatRequest` 将必要上下文传递到 Rust。
  - Rust `kelivo_core::llm` 已提供 `OpenAiAdapter` 与轻包装（google/anthropic/openrouter）但均走 OpenAI 兼容协议。
- 外部依赖与环境：不新增三方依赖；遵循 `docs/rust_frb_build.md` 的 FRB 配置与生成流程。
- 关键约束：
  - 取消操作依赖静态 service 的 inflight 跟踪；需保持该能力。
  - 不改变 FRB 公有函数签名，Flutter 端调用保持不变。
  - 避免持久化/缓存 Provider 配置（请求无状态）。
- 假设 & 待确认：
  - Dart 端可随请求提供 `api_key` 与 `base_url`（见 tasks 中变更）。
  - Dart 端可按需提供 `chat_path`（缺省采用 `/chat/completions`）。

---

## 2. 目标与成功标准（Goals & Exit Criteria）

- 业务/用户目标：恢复“在 Flutter 侧选择任何 Provider/模型均可正常使用”的能力。
- 技术目标：
  - 根据请求元数据即时构建 Provider（OpenAI 兼容），并注册到静态 `LlmService` 后执行。
  - 保持原有取消与流式事件行为；错误语义清晰（含配置缺失/非法）。
- 退出标准：
  - 复现实例中“unknown provider: <模型名>”问题后通过修复消除。
  - 至少验证 3 类 Provider（OpenAI、本地/第三方 OpenAI 兼容、OpenRouter）的流式对话成功返回。
  - 取消在进行中请求时能稳定收到 Cancelled 事件。

---

## 3. 渐进式交付策略（Progressive Strategy）

| Phase | 目标 | 主要内容 | 依赖 | 演示与验收 | 回滚点 |
|------:|------|----------|------|------------|--------|
| 1 | 传参对齐 | Dart 在 `metadata` 填充 `api_key`、`base_url`、`chat_path`、`provider_type` | 现有 UI/配置 | 本地运行，序列化检查 | 还原 Dart 变更 |
| 2 | 动态注册 | Rust 从请求构建 Provider，注册至静态 `LlmService` 并执行 | Phase 1 | 修复前后对照、流式验证 | 恢复为 Mock-only |
| 3 | 健壮性 | 错误分支与取消稳定性测试、日志与文档 | Phase 2 | 用例通过；未知配置可读错误 | 关闭新逻辑开关（回落 Mock） |

---

## 4. 方案概要（Solution Overview）

- 设计思路：
  - 保留一个静态 `LlmService` 仅承担“请求跟踪/取消”职责；在每次请求到达时，依据 `FrbChatRequest.metadata` 动态构造一个 `OpenAiAdapter`，以“当前请求的 provider_id”覆盖注册到该 service，然后立即发起 `chat/chat_stream`。这样既满足无状态（不持久缓存配置），又不破坏现有取消语义。
  - endpoint 计算：`endpoint = join(base_url, chat_path_or_default)`；默认 `chat_path = /chat/completions`。
  - provider_id：返回给 Dart 的 `provider_id` 必须与 Flutter 侧实际选择的 `config.id` 一致（用于 UI 展示与统计），因此在构造 `OpenAiAdapter` 时将 `provider_id` 设为 `config.id`，而不是固定 `openai/google/...`。
- 影响面：
  - Rust：`rust/api/src/api/llm.rs`、（可选）`rust/core/src/llm/mod.rs` 的轻微增强（无需改动接口）。
  - Dart：`lib/core/services/api/chat_api_service.dart` 中 `_createRustRequest/_buildRustMetadata` 扩充。
  - 文档：`docs/rust_frb_build.md` 补充 FRB 与运行时参数约定。
- 兼容性/降级：
  - 若 `api_key/base_url` 缺失则直接在 Rust 侧返回明确错误；Dart 端展示“Rust LLM error: ...”。
  - 通过 Feature Flag 可在必要时回落到 Mock（不属于本次实现，但保持能力）。
- 可观测性：
  - 避免记录明文 `api_key`；如需日志，仅记录 provider_id、endpoint 主机与错误文案。

- 技术栈（Tech Stack）：
  - Flutter（Provider 状态管理，既有代码保持不变）
  - flutter_rust_bridge v2（现有）
  - Rust `kelivo_core::llm::openai::OpenAiAdapter`（统一 OpenAI 兼容协议）

---

## 5. 模块与调用关系（Modules & Flows）

- 模块清单（新/改/复用）
  | 模块 | 职责 | 新增/修改/复用 | 外部接口 |
  |------|------|----------------|----------|
  | `rust/api/src/api/llm.rs` | 解析请求元数据、构造并注册 Provider、转发表达式/流 | 修改 | FRB: `llm_chat`, `llm_chat_stream`, `llm_cancel` |
  | `rust/core/src/llm/openai.rs` | OpenAI 兼容实现 | 复用 | HTTP + SSE |
  | `lib/core/services/api/chat_api_service.dart` | 构造 `FrbChatRequest` 与 `metadata` | 修改 | FRB 调用 |

- 核心调用链（流式）：
  Dart UI → ChatApiService._createRustRequest → FRB.llmChatStream → Rust llm.rs: build_provider_from_request → service.register_provider + set_default → service.chat_stream → SSE 事件编码 → Dart 解析/渲染

---

## 6. 数据与模型（Data & Models）

- 领域实体：沿用 `kelivo_core::llm::{ChatRequest, ChatEvent, ChatResponse}`。
- 元数据约定（FrbChatRequest.metadata）：
  - `provider_id`: Flutter 配置 `config.id`（冗余但作为回退与校验）。
  - `provider_type`: `openai|google|claude|openrouter`（仅用于诊断/统计，不改变协议）。
  - `base_url`: OpenAI 兼容 API 根（不含路径）。
  - `chat_path`: 对话路径（可选，默认 `/chat/completions`）。
  - `api_key`: 访问令牌（必需）。
  - 其他：`header_*`/`body_*` 保留给将来扩展（当前不消费）。

- 请求/响应模型：不改动 FRB 结构；`provider` 字段保持 `config.id`。

---

## 7. 合同与集成（Contracts & Integrations）

- FRB 接口维持：
  | 名称 | 通信 | 请求 | 响应/负载 | 错误语义 |
  |------|------|------|-----------|----------|
  | `llm_chat_stream` | FRB | `FrbChatRequest{provider, model, metadata...}` | SSE JSON: `Delta/Usage/Completed/Error/Cancelled` | 缺参/非法：Error(String) |

- 失败与降级：
  - 缺少 `api_key` 或 `base_url` → `Error("missing api_key/base_url")`。
  - 非法 `base_url` 或拼接 endpoint 失败 → `Error("invalid endpoint: ...")`。
  - 上游 4xx/5xx → `Error("openai chat failed: ...")`（保持 core 约定）。

---

## 9. 校验与验收（Verification & Acceptance）

- 测试层次：
  - 单元：`build_provider_from_request` 元数据解析与 endpoint 拼接；provider_id 回传一致性。
  - 集成：本地对接兼容端点（或 WireMock/内网代理），验证流式与取消。
  - 回归：Flutter 端模型切换、UI 展示的 provider 标识与历史记录写入。
- 关键用例：
  - 选择 OpenAI 官方 + 第三方 OpenAI 兼容 + OpenRouter 各 1 种配置，流式回答成功。
  - 取消：在收到若干 Delta 后取消，请求被取消且不再产生事件。
  - 异常：`api_key` 缺失、`base_url` 缺失、endpoint 无法连接，均返回可读错误。

---

## 10. 性能与资源（Performance & Footprint）

- 关键路径：请求构造、SSE 流处理。
- 目标：与 Mock 相比，新增动态注册不引入显著开销（注册为 O(1) HashMap 覆盖）。
- 主要优化点：必要时可对相同 `provider_id` 做轻量缓存覆盖（当前已覆盖式注册，无额外增长）。

---

## 11. 安全与隐私（Security & Privacy）

- `api_key` 仅用于构造 HTTP 头；不写入日志/存储，不经 FRB 外泄。
- MCP（context7）密钥/端点通过环境变量提供（开发/CI 侧）：`MCP_CONTEXT7_URL`, `MCP_CONTEXT7_TOKEN`（不提交到仓库）。

---

## 12. 观测与运维（Observability & Ops）

- 日志：仅在 Rust 端记录错误语义与 provider_id、host；不打印敏感信息。
- 告警：留给上层（如 UI 层 Toast/SnackBar），不在 Rust 内置。

---

## 13. 影响评估（Impact & Change List）

- 改动清单：
  - `rust/api/src/api/llm.rs`: 移除静态 Mock-only 流程，引入按请求注册。
  - `lib/core/services/api/chat_api_service.dart`: 在 `_createRustRequest` 写入 `api_key/chat_path`。
  - `docs/rust_frb_build.md`: 更新运行时参数说明。
- 兼容性：FRB 接口不变；Dart 端新增 metadata 键向后兼容。

---

## 14. 迁移与回滚（Migration & Rollback）

- 切换计划：按 Phase 顺序合入；每步均可单独回滚（Dart 恢复旧元数据、Rust 恢复 Mock-only）。
- 回滚触发：线上出现系统性错误（大面积 Error/取消失效）时，回退到上一个稳定提交或关闭 USE_RUST_LLM。

---

## 15. 发布与交付（Release & Delivery）

- 分支策略：`spec/rust-llm-provider-fix`；遵循 `specify/Git-Flow.md`。
- CI/CD：合入后跑 `flutter_rust_bridge_codegen` 与 Dart 静态检查/测试；不引入新依赖。

---

## 16. 风险与权衡（Risks & Trade-offs）

| 风险 | 影响 | 可能性 | 缓解 | 回滚触发 |
|------|------|--------|------|----------|
| 取消语义破坏 | 无法取消流 | 低 | 复用静态 service 仅做 inflight | 出现取消不生效 |
| Endpoint 拼接错误 | 请求失败 | 中 | 明确默认 `chat_path` + 单测 | 频繁 invalid endpoint |
| token 泄露风险 | 安全问题 | 低 | 不记录/不持久化 | 日志发现敏感信息 |

---

## 17. 代码组织与约定（Code Map & Conventions）

- 路径与命名：对齐现有 `rust/api/src/api/llm.rs`、`rust/core/src/llm/*`，Dart 改动集中 `lib/core/services/api/chat_api_service.dart`。

