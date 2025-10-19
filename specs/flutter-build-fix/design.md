# 🧠 design.md — Flutter Build Fix

## 0. 元信息（Meta）

- Feature / Bug 名称：`Flutter build fails after FRB refactor`
- Spec 路径：`specs/flutter-build-fix/design.md`
- 版本 / 日期：`v1.0 · 2025-10-19`
- 关联：`requirements.md`、`tasks.md`、`README.md`
- 所有者 / 评审人：`Codex` / `待定`
- 范围声明（Scope / Non-goals）
  - In Scope：修正 `lib/src/rust/mock_api.dart` 以匹配最新 FRB 生成代码；确保 `main.dart` 在 mock 模式下可编译；执行 analyzer/test 快速验证。
  - Out of Scope：修改 Rust 端实现、调整 FRB 生成配置、引入新的依赖或 UI 变更。

---

## 1. 项目基线与约束（Baseline & Constraints）

- 架构与平台：Flutter（Provider 状态管理）+ Rust FFI（flutter_rust_bridge v2.11.1），mock/real 通过 `RustLib.init` 与 `RustLib.initMock` 选择。
- 模块边界与现有约定：所有 Rust 绑定位于 `lib/src/rust/`; mock 实现需实现 `RustLibApi`；包名区分大小写（`Kelivo`）。
- 外部依赖与环境：依赖资产均已生成，无需额外下载；工具链为 Flutter stable + dart analyzer。
- 关键约束：不可调整 `pubspec.yaml` 包名；需保持 mock 行为 deterministic；不得破坏现有 FRB 生成文件（自动生成不手改）。
- 假设 & 待确认：`flutter_rust_bridge_for_generated.dart` 提供的 `PlatformInt64` 即为 analyzer 预期；mock 代码仅在 `USE_RUST=false` 下使用。

---

## 2. 目标与成功标准（Goals & Exit Criteria）

- 业务/用户目标：恢复开发者在 mock 模式下的快速启动能力，避免阻塞移动端迭代。
- 技术目标：mock API 与 `RustLibApi` 接口 100% 对齐，`flutter analyze` / `flutter test` 无编译错误。
- 退出标准：CI 等价检查（analyze/test）通过；`main.dart` 对 `MockRustLibApi` 的注入类型安全。

---

## 3. 渐进式交付策略（Progressive Strategy）

| Phase | 目标 | 主要内容 | 依赖 | 演示与验收 | 回滚点 |
|------:|------|----------|------|------------|--------|
| 1 | 恢复类型定义 | 修正 imports、引入 `llm_types.dart`、移除废弃 API | 现有生成文件 | `flutter analyze` 仅限 `mock_api.dart` 通过 | git revert 本次改动 |
| 2 | 校验应用入口 | 确认 `main.dart` 能初始化 mock；运行 analyze/test | Phase1 完成 | `flutter analyze`、`flutter test --concurrency 1` | 切回 Phase1 状态|

---

## 4. 方案概要（Solution Overview）

- 设计思路：保持 mock 层作为 FRB 的轻量回退实现，只在单文件内完成签名同步；使用常量/占位字符串满足新字段；以 import 校准解决包名大小写问题。
- 影响面：仅影响 `lib/src/rust/mock_api.dart` 与 `main.dart` 类型检查路径；无数据库/网络副作用。
- 兼容性/降级：若未来 Rust 实现就绪，可继续通过 `kUseRust` 切换；mock 仍提供最小功能。
- 可观测性：无需新增日志；沿用现有控制台打印。

- 技术栈（Tech Stack）：
  - 架构模式：Flutter multi-provider + Rust FFI。
  - 状态管理：Provider。
  - 网络层：未触及（mock 返回静态数据）。
  - 本地存储：未触及。
  - UI 框架/样式：Flutter Material。

- 分层视图（示意）：
  ```
  Presentation → Services (Chat/Backup) → Rust Binding (Real or Mock)
  ```

---

## 5. 模块与调用关系（Modules & Flows）

- 模块清单
  | 模块 | 职责 | 新增/修改/复用 | 外部接口 |
  |------|------|----------------|----------|
  | `lib/src/rust/mock_api.dart` | Mock 实现 Rust API | 修改 | 实现 `RustLibApi` |
  | `lib/main.dart` | 应用入口，注入 mock/real | 复用（类型校验） | `RustLib.initMock` |

- 核心调用链：`main.dart` → `RustLib.initMock(api: MockRustLibApi())` → Provider 服务通过 `RustLib.instance.api` 调用。
- 状态机：不适用。
- 模块按层划分：
  - Presentation：`lib/features/...` (未改动)
  - Data / Integration：`lib/src/rust/mock_api.dart`

---

## 6. 数据与模型（Data & Models）

- 领域实体：`FrbConversation`, `FrbMessage`, `FrbToolEvent`, `FrbChatResponse`, `BackupZipEntryInput` 等，均来自生成文件。
- DTO ↔ Domain 映射
  | 字段 | DTO | Domain | 默认值 | 兼容性 |
  |------|-----|--------|--------|--------|
  | `requestId` | `FrbChatResponse` | ChatService 消费 | `'mock-request'` | mock-only |
  | `providerId` | `FrbChatResponse` | ChatService | `'mock-provider'` | mock-only |
  | `outputText` | `FrbChatResponse` | ChatService | `'mock response'` | mock-only |
  | `usage` | `FrbChatUsage` | ChatService | zero tokens | 与真实响应结构一致 |
- 存储与迁移：无。
- 隐私与敏感：mock 仅包含静态文本，不含敏感信息。

---

## 7. 合同与集成（Contracts & Integrations）

- 接口/事件清单
  | 名称 | 通信方式 | 请求 | 响应/负载 | 鉴权/幂等 | 错误语义 |
  |------|----------|------|-----------|-----------|----------|
  | `RustLibApi` methods | 方法调用 | Dart → Mock | 同步/Future | 不涉及 | 返回 `Future`、`Stream`，抛出 `Exception` 时写日志 |
- 失败与降级：若 mock 抛异常则保持当前打印 + rethrow 行为。
- 灰度与回滚：通过 git revert 单文件回滚。
- 接口示例：不适用。

---

## 8. UI 与交互（UI/UX & A11y）

- 不涉及 UI 改动；现有页面/组件保持不变。

---

## 9. 校验与验收（Verification & Acceptance）

- 测试层次：
  - 单元：无新增。
  - 集成：`flutter analyze`、`flutter test --concurrency 1`。
- 关键用例表
  | 编号 | 场景 | 前置 | 步骤 | 期望 | 验收方式 |
  |-----:|------|------|------|------|----------|
  | 1 | mock init | `USE_RUST=false` | 启动应用或运行 analyzer | 无编译错误 | 本地命令
  | 2 | mock chat call | 调用 `RustLib.instance.api.crateApiLlmLlmChat` 在测试中 | 返回 mock 数据 | 单元/手测（可选） |
- 手测策略：运行应用（可选），至少完成 analyzer + test。

---

## 10. 性能与资源（Performance & Footprint）

- 关键路径：无；mock 返回常量。
- 目标：不增加编译时间显著。
- 主要优化点：无。

---

## 11. 安全与隐私（Security & Privacy）

- Mock 数据无敏感信息；不引入网络调用；日志保持现状。

---

## 12. 观测与运维（Observability & Ops）

- 指标与埋点：无新增。
- 日志与聚合：延续 `_readFileAsString` 的错误打印。
- 告警阈值：不适用。

---

## 13. 影响评估（Impact & Change List）

- 改动清单：`lib/src/rust/mock_api.dart` 更新，可能调整 `main.dart` import 顺序（若需要）。
- 兼容性：无破坏性；mock 路径恢复工作。
- 协作依赖：无。

---

## 14. 迁移与回滚（Migration & Rollback）

- 数据迁移：不适用。
- 配置/版本控制：保持 `specs/flutter-build-fix` 分支；如需回滚直接 `git checkout -- lib/src/rust/mock_api.dart`。
- 切换计划：合并后立即生效，无阶段切换。

---

## 15. 发布与交付（Release & Delivery）

- 分支策略：遵循 `specs/<feature>` 分支；提交完成后合并上游流程按项目约定执行。
- CI/CD 要点：确保在提交前运行 analyzer/test；CI 不需额外配置。
- 发布清单：N/A（无应用商店发布）。

---

## 16. 风险与权衡（Risks & Trade-offs）

| 风险 | 影响 | 可能性 | 缓解 | 回滚触发 |
|------|------|--------|------|----------|
| Mock 返回字段缺失 | Chat 页面崩溃 | 低 | 使用默认值并对照新类型 | 观察到异常日志 |
| 未来 FRB 再次变更签名 | 再次编译失败 | 中 | 在生成后立即同步 mock | analyze 报错 |

---

## 17. 代码组织与约定（Code Map & Conventions）

- 目录与命名：保持 `lib/src/rust/…`；包导入使用 `package:Kelivo/...`；常量遵循 lowerCamelCase。
- 注释与文档：复杂逻辑处添加单行注释说明（例如 mock 返回结构）。

---

## 18. 评审清单（Review Checklist）

- [ ] 与 `requirements.md` 对齐
- [ ] 每个 Phase 可运行/可测试/可回滚
- [ ] 兼容性与风险明确

---

## 19. 附录（Appendix）

- 无其他补充；若后续需要扩展 mock 行为，可在此记录。
