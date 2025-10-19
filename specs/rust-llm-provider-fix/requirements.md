# 📄 requirements.md — Rust Core LLM Provider Fix

## 1. Introduction（需求背景）

### 1.1 问题陈述

当前，Flutter 客户端具备支持多种兼容 OpenAI API 的 LLM 服务的能力，但后端的 Rust 核心模块存在逻辑缺陷，导致该功能完全失效。Rust 模块使用了一个静态的、硬编码的 `MockProvider`，完全忽略了从 Flutter 客户端传递过来的动态 Provider 配置。

本次任务的目标是修复 Rust 核心模块，使其能够正确响应 Flutter 侧的动态配置，恢复并解锁对多 Provider 的支持。

### 1.2 Flutter 侧现有逻辑详解

为了明确 Rust 侧需要满足的需求，必须首先理解 Flutter 侧已经实现的、完整的模型选择和对话发起逻辑。该逻辑完全在客户端本地进行，核心是允许用户动态配置和选择不同的服务提供商（Provider）。

#### 1.2.1 数据的来源与存储 (`core/providers/settings_provider.dart`)

- **Provider配置 (`ProviderConfig`)**: 这是最核心的数据结构。每个 `ProviderConfig` 对象代表一个 AI 服务提供商（例如 "OpenAI"、"Groq" 或用户自定义的任何兼容服务）。它包含了以下关键信息：
  - `id`: 唯一标识符。
  - `name`: 显示名称。
  - `apiKey`: 服务的 API 密钥。
  - `baseUrl`: **服务的 API 根地址** (例如 `https://api.openai.com/v1`)。这是支持不同服务的关键。
  - `models`: 一个字符串列表，包含了这个 Provider 下可用的模型 ID (例如 `['gpt-4', 'gpt-3.5-turbo']`)。
  - `modelOverrides`: 一个 Map，允许为特定模型 ID 覆盖显示名称、能力（如是否支持工具）等元数据。

- **数据持久化**:
  - 所有的 `ProviderConfig` 对象都存储在一个 Map `_providerConfigs` 中。
  - 这个 Map 以及当前选中的模型 ID (`_selectedModelKey`) 等信息，都会被序列化成 JSON 字符串，并使用 `shared_preferences` 库存储在设备本地。
  - 应用启动时，`SettingsProvider` 会从 `shared_preferences` 加载这些配置到内存中。

- **模型的来源**:
  - **主要是“写死”和“用户自定义”**。应用内置了一些默认 Provider 配置，同时用户可以通过 UI 添加全新的 Provider，并**手动填写**其下的模型 ID 列表。
  - **不存在**一个从服务器“下发”模型列表的通用机制。模型的可用性依赖于用户在 Provider 配置中的手动输入。

#### 1.2.2 UI 与选择流程

- **触发选择 (`features/home/pages/home_page.dart`)**: 在主聊天界面，用户点击顶部当前模型的名称时，会调用 `showModelSelectSheet(context)` 方法来弹出模型选择列表。

- **模型选择列表 (`features/model/widgets/model_select_sheet.dart`)**: 
  - 该列表会从 `SettingsProvider` 中读取所有已启用的 Provider 及其下的模型列表。
  - 列表中的模型按其所属的 Provider 进行**分组**展示，并提供收藏、搜索和快捷跳转功能。

- **完成选择**: 
  - 用户点击选择一个模型后，`SettingsProvider` 的 `setCurrentModel(providerKey, modelId)` 方法会被调用，将用户的选择持久化到 `shared_preferences`。

#### 1.2.3 对话发起 (`features/home/pages/home_page.dart`)

- **使用所选模型**: 
  1. 当用户发送消息时，`_sendMessage` 方法首先从 `SettingsProvider` 获取当前选中的 `providerKey` 和 `modelId`。
  2. 如果已选择模型，应用会用 `providerKey` 从 `SettingsProvider` 中获取到完整的 `ProviderConfig` 对象。
  3. 最后，这个 `ProviderConfig` 对象（包含了 `baseUrl`, `apiKey` 等）和 `modelId` 以及聊天内容一起，被打包成 `FrbChatRequest` 对象，**发送给 Rust 核心模块**。

### 1.3 结论

Flutter 侧的设计已经完全支持多 Provider，并将每次请求所需的全部上下文（`baseUrl`, `apiKey` 等）都打包发送给了 Rust。Rust 侧的改造必须基于“**请求是无状态且自包含的**”这一事实，废弃现有静态逻辑，改为按需处理。

---

## 2. 需求描述（Requirements）

- **R1:** Rust 核心模块必须能够解析从 Flutter 客户端传入的每个聊天请求中的 `providerKey`, `modelId`, `baseUrl`, 和 `apiKey`。
- **R2:** Rust 核心模块必须根据请求中的 `baseUrl` 动态构建 API 客户端，而不是使用任何静态或硬编码的 API 端点。
- **R3:** 现有的 `OpenAiAdapter` 应被用于处理所有兼容 OpenAI 协议的请求，使其成为一个通用的、按需实例化的适配器。
- **R4:** 修复必须是无状态的。Rust 层不应存储或缓存 Provider 配置。每个请求都应被视为独立的、自包含的事务。
- **R5:** 解决方案必须优雅地处理无效配置（如格式错误的 `baseUrl`）的错误情况，并通过 FFI 边界向上传递有意义的错误信息。

---

## 3. 分阶段开发策略（Phased Development Strategy）

| Phase | 标题 | 简要说明 |
|-------|------|----------|
| Phase 1 | 核心逻辑重构 | 移除 Rust 代码中的静态 `LlmService`，并实现按需动态实例化 Provider 的核心逻辑。 |
| Phase 2 | 功能验证与测试 | 添加单元测试和集成测试，确保动态 Provider 逻辑在各种情况下（包括错误情况）都能正常工作。 |

---

## 4. Requirements（详细需求）

### Phase 1: 核心逻辑重构

#### Requirement R1-R4: 动态 Provider 实例化

User Story:
- 作为一名开发者，我希望 Rust 核心模块能够为每个传入的聊天请求动态创建一个 LLM provider 实例，以便系统可以与用户在 Flutter 应用中配置的任何 OpenAI 兼容 API 进行通信。

Acceptance Criteria:
- [AC1] `rust/api/src/api/llm.rs` 中的静态 `LLM_SERVICE` 被完全移除。
- [AC2] `llm_chat_stream` 函数被重构，以在函数体内根据 `FrbChatRequest` 中的 `baseUrl` 和 `apiKey` 创建一个 `OpenAiAdapter` 实例。
- [AC3] 新创建的 `OpenAiAdapter` 实例被用于处理当前的聊天请求。
- [AC4] 整个处理流程是无状态的；函数执行完毕后，不应有残留的 Provider 实例。

### Phase 2: 功能验证与测试

#### Requirement R5: 错误处理与健壮性

User Story:
- 作为一名开发者，我希望当从客户端接收到无效的 Provider 配置时，Rust 代码能够优雅地失败并返回一个明确的错误，以便客户端可以向用户显示有用的反馈。

Acceptance Criteria:
- [AC1] 如果 `baseUrl` 格式不正确或无法访问，`OpenAiAdapter` 的创建会返回一个 `Result::Err`。
- [AC2] 这个错误被捕获并转换为一个字符串，通过 FFI 边界传递回 Flutter 客户端。
- [AC3] 添加了相应的单元测试来验证错误处理路径。

---

## 5. Non-functional & Cross-cutting（非功能与横切）

### 技术架构与代码规范
- 严格遵循现有的 Rust 代码风格和模块化结构。
- 避免引入新的第三方依赖，除非对于解决核心问题是绝对必要的。

### 错误处理与用户体验
- 从 Rust 返回的错误信息应足够清晰，以便于调试，但又不能暴露敏感的内部实现细节。

### 性能与安全
- 动态创建 HTTP 客户端可能会引入性能开销。初步实现将接受此开销，但应在 `design.md` 中探讨潜在的优化方案（如客户端缓存）。
- `apiKey` 必须作为敏感信息处理，仅在内存中使用，不得记录到日志中。
