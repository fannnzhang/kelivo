# 📄 requirements.md — Google Auth JWT 签名迁移至 Rust

## 1. Introduction（需求背景）

- **当前实现**: 目前，应用通过 `google_service_account_auth.dart` 文件与 Google 服务账户进行认证。其中，认证流程的关键步骤——创建和签名 JWT (JSON Web Token)——是使用纯 Dart 的 `jose` 包在应用层完成的。
- **迁移目标**: 为了提升应用的整体性能、加强安全性和统一核心逻辑，本项目计划将此 JWT 签名功能从 Dart 下沉到 Rust 实现。Rust 在加密计算和内存安全方面的优势，使其成为执行此类安全敏感型任务的理想选择。

---

## 2. 需求描述（Requirements）

- 将 `google_service_account_auth.dart` 中用于生成 Google API 访问令牌的 JWT 签名逻辑，从 Dart 的 `jose` 包实现迁移至由 Rust 编写并通过 `flutter_rust_bridge` 暴露的本地函数。

---

## 3. 分阶段开发策略（Phased Development Strategy）

此项目作为一个独立的、内聚的功能迁移，将通过单一阶段完成。

| Phase | 标题 | 简要说明 |
|-------|------|----------|
| Phase 1 | 功能实现与集成 | 开发 Rust 函数完成 JWT 签名，并在 Dart 层进行替换和集成测试。 |

---

## 4. Requirements（详细需求）

### Phase 1: 功能实现与集成

#### Requirement 1.1: 创建并暴露 Rust JWT 签名函数

User Story:
- 作为一名开发者，我希望在 Rust 核心库中实现一个 JWT 签名函数，并通过 FRB 将其暴露给 Dart，以便在应用中调用高性能且安全的本地加密能力。

Acceptance Criteria:
- [x] 在 Rust 端创建一个新的公开函数，例如 `create_google_auth_jwt`。
- [x] 函数应接受 `private_key` (PEM 格式字符串), `client_email`, `token_uri`, 和 `scopes` (字符串列表) 作为输入参数。
- [x] 函数内部必须构建一个与当前 Dart 实现完全相同的 JWT Claims Set，包含 `iss`, `scope`, `aud`, `iat`, `exp` 字段。
- [x] 必须使用 `RS256` 算法和传入的私钥对 Claims Set 进行签名。
- [x] 函数成功时需返回签名后的 JWT（Compact Serialization 格式的字符串）。
- [x] 函数必须通过 `flutter_rust_bridge` 正确生成并在 Dart 端可用。

#### Requirement 1.2: 在 Dart 层集成 Rust 函数

User Story:
- 作为一名开发者，我希望在 `GoogleServiceAccountAuth` 类中，用对 Rust 本地函数的调用来替换原有的 Dart JWT 签名逻辑，以便将计算密集型任务转移到原生层。

Acceptance Criteria:
- [x] 在 `google_service_account_auth.dart` 的 `getAccessToken` 方法中，移除所有对 `jose` 包（如 `JsonWebSignatureBuilder`）的调用。
- [x] 在上述位置，添加对新 FRB 生成的 Rust 函数 `create_google_auth_jwt` 的调用，以获取 `assertion` 字符串。
- [x] 整个 `getAccessToken` 方法的外部行为（包括网络请求和缓存）必须保持不变。
- [x] 迁移后，Google 服务账户的认证流程必须能成功完成，并获取到有效的 `access_token`。
- [x] （可选）如果 `jose` 包不再被项目其他地方使用，应考虑从 `pubspec.yaml` 中移除。

---

## 5. Non-functional & Cross-cutting（非功能与横切）

### 技术架构与代码规范
- Rust 实现应遵循项目已有的代码风格和目录结构。
- 应优先选用社区广泛认可、维护活跃的 Rust Crate 来处理 JWT 和加密操作（例如 `jsonwebtoken`, `ring`）。
- 新增的 Rust 代码必须通过 `cargo fmt` 和 `cargo clippy` 的检查。

### 错误处理与用户体验
- Rust 函数在遇到错误（如私钥格式无效、签名失败）时，必须通过 `Result` 类型将错误信息清晰地传递回 Dart 调用端。
- Dart 端必须妥善处理来自 Rust 的错误，例如通过抛出异常来中断认证流程，防止静默失败。
- 本次迁移不应引入任何面向用户的 UI 或体验变更。

### 性能与安全
- 私钥等敏感信息在传递和使用过程中应被安全地处理。
- 迁移后的 JWT 签名性能应优于或等于原有的 Dart 实现。建议编写基准测试来量化性能提升。
