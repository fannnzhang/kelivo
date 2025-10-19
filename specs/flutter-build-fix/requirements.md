# 📄 requirements.md — Flutter Build Fix

## 1. Introduction（需求背景）

- Recent Rust FRB upgrades reorganized generated Dart bindings under `lib/src/rust/api/…` and changed several data models. The Flutter side still ships the legacy mock implementation (`lib/src/rust/mock_api.dart`), causing analyzer and build failures when `USE_RUST` is disabled and the mock path is used. This spec restores compile health in alignment with the repository README and existing Rust integration strategy.

---

## 2. 需求描述（Requirements）

- The Flutter tree must compile with `flutter analyze` and `flutter test` using the new FRB-generated modules.
- `MockRustLibApi` shall implement `RustLibApi` exactly as defined in `lib/src/rust/frb_generated.dart` (v2.11.1 output) with correct return payloads and signatures.
- Dart imports within `lib/src/rust/mock_api.dart` shall target the capitalized package name `Kelivo` and the new `api/` module layout.
- Default mock data returned by the API shall reflect the new FRB types (`FrbDbSnapshot`, `FrbChatResponse`, etc.) so downstream callers continue to work during mock-driven development.
- No additional runtime dependencies or architectural deviations may be introduced; changes must respect the current module structure described in `README.md`.

---

## 3. 分阶段开发策略（Phased Development Strategy）

| Phase | 标题 | 简要说明 |
|-------|------|----------|
| Phase 1 | Align mock API sources | Update imports, type usage, and method signatures in `lib/src/rust/mock_api.dart` to match regenerated bindings. |
| Phase 2 | Validate build health | Run analyzer/tests and ensure `main.dart` mock initialization compiles without warnings or errors. |

---

## 4. Requirements（详细需求）

### Phase 1: Align mock API sources

#### Requirement 1: Update imports and dependencies

User Story:
- 作为 Flutter 开发者，我希望 mock API 文件引用正确的包路径和 FRB 类型，这样我在关闭 Rust 实现时依然可以成功编译应用。

Acceptance Criteria:
- Imports in `lib/src/rust/mock_api.dart` use `package:Kelivo/src/rust/...` and include `llm_types.dart` plus `flutter_rust_bridge_for_generated.dart` for `PlatformInt64`.
- No analyzer `uri_does_not_exist` or `implements_non_class` errors remain for the file.

#### Requirement 2: Synchronize method contracts

User Story:
- 作为 开发者，我希望 mock 实现与生成的 `RustLibApi` 接口保持一致，以便热切换 mock/real 时不会引入编译错误。

Acceptance Criteria:
- `MockRustLibApi` overrides every method defined in `RustLibApi` with matching signatures (parameter names, nullability, and return types).
- Outdated methods such as `crateApiDbDbUpsertToolCall` are removed, and new structures (`FrbChatResponse`, `FrbDbInfo`) are instantiated with valid field values.
- Mock outputs supply sensible defaults (e.g., empty lists, zero counts, placeholder strings) to keep existing unit/widget tests deterministic.

### Phase 2: Validate build health

#### Requirement 3: Analyzer and tests green

User Story:
- 作为 CI 工程师，我希望在本地运行快速检查确认 Flutter 树无编译错误，这样 CI 能顺利通过。

Acceptance Criteria:
- `flutter analyze` completes without errors (warnings allowed as per current repo state).
- `flutter test --concurrency 1` completes without compile-time failures (existing flaky warnings allowed, but no new failures introduced).
- Document the executed commands and results in the task log per `tasks.md`.

---

## 5. Non-functional & Cross-cutting（非功能与横切）

### 技术架构与代码规范
- 保持与现有 Flutter/Rust 模块结构一致；禁止引入新的第三方依赖或额外层次。
- 遵循仓库现有的命名约定（包名 `Kelivo`，模块放置于 `lib/src/rust/…`）。

### 错误处理与用户体验
- Mock 返回的错误信息维持可读性，避免抛出未捕获异常；读取文件的 fallback 逻辑保持当前行为。

### 性能与安全
- Mock 路径无需持久化或重 IO 操作；确保不存储敏感数据并继续使用本地文件读取的防护（同当前实现）。
