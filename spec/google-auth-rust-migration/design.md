# 🧠 design.md — Google Auth JWT 签名迁移至 Rust 方案设计

## 0. 元信息（Meta）

- **Feature 名称**：`feat(rust): migrate-google-auth-jwt-signing`
- **Spec 路径**：`spec/google-auth-rust-migration/design.md`
- **版本 / 日期**：v1.0 · 2025-10-15
- **关联**：`requirements.md`
- **所有者 / 评审人**：Gemini
- **范围声明（Scope / Non-goals）**
  - **In Scope**：将 `google_service_account_auth.dart` 中使用 `jose` 包生成 JWT 的逻辑替换为 Rust 实现。
  - **Out of Scope**：认证流程中的 HTTP 请求、Token 缓存逻辑、以及项目其他部分的任何代码。

---

## 1. 项目基线与约束（Baseline & Constraints）

- **架构与平台**：项目已集成 Flutter Rust Bridge (FRB)，为 Dart 与 Rust 之间的通信提供了现有通道。
- **模块边界与现有约定**：Rust 逻辑应封装在 `rust/src/api/` 目录下的新模块中，并遵循现有的 FRB 函数暴露模式。
- **外部依赖与环境**：Dart 侧依赖 `http` 包进行网络请求。Rust 侧将引入新的加密和 JWT 相关 Crate。
- **关键约束**：新的 Rust 实现必须在所有目标平台（iOS, Android, Web, Desktop）上行为一致。必须使用经过社区安全审计的、生产级的 Rust Crate。

---

## 2. 目标与成功标准（Goals & Exit Criteria）

- **技术目标**：
  1. 移除 Dart `jose` 包在 JWT 签名场景的使用，替换为更安全、更高性能的 Rust 原生实现。
  2. 验证通过 FRB 进行加密操作的可行性与性能优势。
- **退出标准**：
  1. JWT 签名逻辑在 Rust 中实现并通过单元测试。
  2. Dart 代码成功集成 Rust 函数，并通过集成测试，Google 服务账户认证流程可正常工作。
  3. 性能基准测试（可选）表明 Rust 实现不劣于（并优于）Dart 实现。

---

## 3. 渐进式交付策略（Progressive Strategy）

| Phase | 目标 | 主要内容 | 依赖 | 演示与验收 | 回滚点 |
|------:|------|----------|------|------------|--------|
| 1 | 功能实现与集成 | 1. 开发 Rust JWT 签名函数。<br>2. 编写 Rust 单元测试。<br>3. 在 Dart 中替换原有逻辑。<br>4. 运行集成测试验证端到端流程。 | 无 | 演示认证流程成功获取 Token | 切换回使用 `jose` 包的旧代码实现。 |

---

## 4. 方案概要（Solution Overview）

- **设计思路**：本方案采用“精准下沉”策略，仅将计算最密集、安全最敏感的 JWT 签名操作迁移到 Rust。Dart 层继续负责流程编排（如参数准备、HTTP 调用、缓存管理），Rust 层则作为一个纯粹的、无副作用的加密计算单元。这种设计将改动范围降至最低，同时最大化 Rust 的优势。
- **影响面**：
  - **修改**: `lib/core/services/api/google_service_account_auth.dart`
  - **新增**: `rust/src/api/google_auth.rs` (或类似名称)
  - **依赖**: `Cargo.toml` 将新增 `jsonwebtoken`, `chrono`, `serde` 等 crate。
- **兼容性/降级**：无兼容性问题。由于是纯逻辑替换，可通过代码注释或分支轻松回滚到旧实现。

---

## 5. 模块与调用关系（Modules & Flows）

- **模块清单**
  | 模块 | 职责 | 新增/修改/复用 | 外部接口 |
  |------|------|----------------|----------|
  | `google_service_account_auth.dart` | 编排认证流程，调用 Rust 获取 JWT | 修改 | `getAccessToken()` |
  | `rust/src/api/google_auth.rs` | 创建并签名 JWT | 新增 | `create_google_auth_jwt()` |

- **核心调用链**：
  `Dart:getAccessToken()` → `Rust:create_google_auth_jwt()` → `Dart:getAccessToken()` → `http.post()`

---

## 6. 数据与模型（Data & Models）

- **领域实体**：JWT Claims
  - 在 Rust 函数内部定义，用于序列化为 JWT payload。
  ```rust
  #[derive(Debug, Serialize, Deserialize)]
  struct Claims {
      iss: String,   // issuer
      scope: String, // space-separated scopes
      aud: String,   // audience
      iat: i64,      // issued at (epoch seconds)
      exp: i64,      // expiration (epoch seconds)
  }
  ```
- **隐私与敏感**：`private_key` 作为字符串在 Dart 和 Rust 之间传递。在 Rust 端，它仅在函数作用域内用于解码和签名，不会被存储或泄露。

---

## 7. 合同与集成（Contracts & Integrations）

- **接口/事件清单**
  | 名称 | 通信方式 | 请求 (参数) | 响应/负载 | 鉴权/幂等 | 错误语义 |
  |------|----------|------|-----------|-----------|----------|
  | `create_google_auth_jwt` | FRB (Rust) | `private_key: String`, `client_email: String`, `token_uri: String`, `scopes: Vec<String>` | `Result<String, String>` | N/A | 返回包含错误信息的 `Err` |

- **接口示例（FRB）**
  ```rust
  // In rust/src/api/google_auth.rs
  pub fn create_google_auth_jwt(
      private_key_pem: String,
      client_email: String,
      token_uri: String,
      scopes: Vec<String>,
  ) -> anyhow::Result<String> {
      // ... implementation ...
  }
  ```

---

## 9. 校验与验收（Verification & Acceptance）

- **测试层次**：单元测试 (Rust), 集成测试 (Flutter)
- **关键用例表**
  | 编号 | 场景 | 前置 | 步骤 | 期望 | 验收方式 |
  |-----:|------|------|------|------|----------|
  | 1.1 | Rust 单元测试 | 提供有效的 PEM 私钥和参数 | 调用 `create_google_auth_jwt` | 返回一个格式正确的 JWT 字符串 | `cargo test` |
  | 1.2 | Rust 单元测试 | 提供无效的 PEM 私钥 | 调用 `create_google_auth_jwt` | 返回包含“InvalidKey”等信息的 `Err` | `cargo test` |
  | 2.1 | Flutter 集成测试 | Mock HTTP Client, 提供有效凭证 | 调用 `GoogleServiceAccountAuth.getAccessToken` | 函数成功返回 mock token，无异常 | `flutter test` |

---

## 11. 安全与隐私（Security & Privacy）

- **数据分类与保护**：`private_key` 被视为高度敏感数据。它在内存中以字符串形式存在，仅在需要时传递给 Rust 函数，Rust 函数处理后立即销毁其作用域内的副本，符合最小权限和最短生命周期原则。

---

## 16. 风险与权衡（Risks & Trade-offs）

| 风险 | 影响 | 可能性 | 缓解 | 回滚触发 |
|------|------|--------|------|----------|
| Rust Crate 存在安全漏洞 | 可能导致私钥泄露或签名被伪造 | 低 | 选择广泛使用、积极维护且经过审计的 Crate (如 `jsonwebtoken`) | 发现严重安全公告 |
| 性能不达预期 | 影响认证速度 | 极低 | Rust 的加密性能通常远超 Dart。编写基准测试验证。 | 性能严重差于 Dart |

---

## 17. 代码组织与约定（Code Map & Conventions）

- **目录与命名**：
  - 新增 Rust 文件：`rust/src/api/google_auth.rs`
  - 在 `rust/src/api/mod.rs` 中声明新模块 `pub mod google_auth;`
  - Dart 调用侧保持不变。
