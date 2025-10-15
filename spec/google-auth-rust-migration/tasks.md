# 🛠 tasks.md — Google Auth JWT 签名迁移任务清单

## 分阶段开发策略（Phases Overview）

- **Phase 1: Rust 实现与单元测试** (在 Rust 端独立完成 JWT 签名函数的开发与验证)
- **Phase 2: Dart 集成与测试** (将 Rust 函数集成到 Dart 代码中，并进行端到端验证)
- **Phase 3: 清理与收尾** (代码格式化、移除无用依赖、最终评审)

---

## Phase 1: Rust 实现与单元测试

- [x] **1. 环境搭建与模块定义**
  - **Summary**: 在 Rust 项目中创建新模块并添加所需依赖。
  - **Files**:
    - `rust/Cargo.toml`
    - `rust/src/api/mod.rs`
    - `rust/src/api/google_auth.rs`
  - **Changes**:
    - 在 `Cargo.toml` 的 `[dependencies]` 中添加 `jsonwebtoken`, `serde`, `chrono`, `anyhow`。
    - 在 `rust/src/api/mod.rs` 中添加 `pub mod google_auth;`。
    - 创建 `rust/src/api/google_auth.rs` 文件。
  - **Requirements**: `R1.1`
  - **Acceptance**: 项目可以成功编译。

- [x] **2. 实现 JWT 签名核心逻辑**
  - **Summary**: 编写 `create_google_auth_jwt` 函数，实现 JWT 的创建和签名。
  - **Files**:
    - `rust/src/api/google_auth.rs`
  - **Changes**:
    - 定义 `Claims` 结构体。
    - 实现 `create_google_auth_jwt` 函数，包含 Claims 构建、`RS256` 签名等核心逻辑。
    - 使用 `anyhow::Result` 进行错误处理。
  - **Requirements**: `R1.1`
  - **Acceptance**: 函数逻辑完整，能够根据输入生成 JWT 字符串或返回错误。

- [x] **3. 编写 Rust 单元测试**
  - **Summary**: 为 `create_google_auth_jwt` 函数编写单元测试，覆盖成功和失败场景。
  - **Files**:
    - `rust/src/api/google_auth.rs` (在 `#[cfg(test)]` 模块下)
  - **Changes**:
    - 添加一个测试用例，使用有效的密钥和参数，断言函数返回 `Ok(jwt_string)`。
    - 添加一个测试用例，使用无效/格式错误的密钥，断言函数返回 `Err`。
  - **Requirements**: `R1.1`
  - **Acceptance**: `cargo test` 执行通过。

---

## Phase 2: Dart 集成与测试

- [x] **1. 生成 Bridge 代码**
  - **Summary**: 运行 FRB 代码生成器，使新的 Rust 函数在 Dart 中可用。
  - **Changes**: FRB 生成的 `frb_generated.*.dart` 文件将被更新。
  - **Requirements**: `R1.1`
  - **Acceptance**: 在 Dart 代码中可以找到并调用 `createGoogleAuthJwt` 函数。

- [x] **2. 替换 Dart 实现**
  - **Summary**: 修改 `GoogleServiceAccountAuth` 类，用 Rust 函数调用替换旧的 `jose` 包逻辑。
  - **Files**:
    - `lib/core/services/api/google_service_account_auth.dart`
  - **Changes**:
    - 在 `getAccessToken` 方法中，删除 `JsonWebSignatureBuilder` 相关代码块。
    - 在原位置添加对 `rust_api.createGoogleAuthJwt(...)` 的 `await` 调用。
    - 添加对 Rust 函数返回的 `Result` 的错误处理逻辑。
  - **Requirements**: `R1.2`
  - **Acceptance**: 代码编译通过，旧逻辑被完全替换。

- [x] **3. 编写/调整集成测试**
  - **Summary**: 验证整个认证流程在替换实现后依然能正常工作。
  - **Files**:
    - `test/...` (相关的集成测试文件)
  - **Changes**:
    - 确保有集成测试覆盖 `GoogleServiceAccountAuth.getAccessToken`。
    - Mock HTTP 请求，断言在给定有效凭证时，函数能走通 Dart-Rust-Dart 的调用并返回预期的 mock token。
  - **Requirements**: `R1.2`
  - **Acceptance**: `flutter test` 中相关的集成测试通过。

---

## Phase 3: 清理与收尾

- [x] **1. 代码格式化与审查**
  - **Summary**: 确保所有新代码符合项目规范。
  - **Files**: 所有修改过的文件。
  - **Changes**:
    - 运行 `dart format . --fix` 和 `cargo fmt`。
    - 运行 `flutter analyze` 和 `cargo clippy` 并修复警告。
  - **Requirements**: `Non-functional`
  - **Acceptance**: 无格式或静态分析问题。

- [x] **2. 移除无用依赖**
  - **Summary**: 如果 `jose` 包不再被使用，将其从项目中移除。
  - **Files**:
    - `pubspec.yaml`
  - **Changes**:
    - 运行 `dart pub remove jose`。
  - **Requirements**: `R1.2`
  - **Acceptance**: `pubspec.yaml` 和 `pubspec.lock` 中不再包含 `jose`。