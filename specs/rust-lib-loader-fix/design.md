# 🧠 design.md — rust_lib_Kelivo Loader 修复

## 0. 元信息（Meta）

- Feature / Bug 名称：`rust_lib_Kelivo Loader Failure`
- Spec 路径：`specs/rust-lib-loader-fix/design.md`
- 版本 / 日期：`v1.0 · 2025-02-14`
- 关联：`requirements.md`、`tasks.md`、`docs/rust_frb_build.md`
- 所有者 / 评审人：`Codex Agent` / `@kelivo-mobile`
- 范围声明（Scope / Non-goals）
  - In Scope：Flutter 端 FRB 加载配置、codegen 同步、启动验证、文档校对。
  - Out of Scope：Rust 业务逻辑实现、FRB API 变更、MCP context7 配置之外的外部依赖。

---

## 1. 项目基线与约束（Baseline & Constraints）

- 架构与平台：Flutter (Provider) + FRB v2，主入口 `lib/main.dart`，Rust crate 位于 `rust/api`（`rust_lib_Kelivo`）。
- 模块边界与现有约定：`lib/src/rust/*` 由 `flutter_rust_bridge_codegen` 生成；加载逻辑在 `RustLib.init()`；`docs/rust_frb_build.md` 定义构建顺序。
- 外部依赖与环境：`flutter_rust_bridge`、`rust_builder`（iOS/macos/Android 脚手架）、MCP `context7` 用于命令远程执行记录。
- 关键约束：iOS/macOS 需静态/动态库名称完全一致；不得破坏 mock 初始化；代码生成文件必须保持自动化流程。
- 假设 & 待确认：若无法现场验证真机运行，需通过 `context7` pipeline 或提供 “To Confirm” 记录与目标日期。

---

## 2. 目标与成功标准（Goals & Exit Criteria）

- 业务/用户目标：恢复在真实 Rust 模式下的稳定启动体验，避免首屏即崩溃。
- 技术目标：统一动态库名称、确保 codegen 与 crate 名同步、完善测试链路。
- 退出标准：`USE_RUST=true` 启动无 `dlopen` 错误；静态分析/单测通过；文档和任务完成度更新；至少一次验证记录（本地或 context7）。

---

## 3. 渐进式交付策略（Progressive Strategy）

| Phase | 目标 | 主要内容 | 依赖 | 演示与验收 | 回滚点 |
|------:|------|----------|------|------------|--------|
| 1 | 修复 loader | 更新默认 `stem` 为 `rust_lib_Kelivo` | 现有 FRB 生成文件 | 输出 diff，模拟运行 | 恢复旧 stem 或 mock-only |
| 2 | 同步生成 & 验证 | 重新 codegen、跑通 `fvm dart analyze` & `fvm flutter test`、记录 `context7` | Flutter/Rust toolchain | 测试日志、`flutter run --dart-define=USE_RUST=true` | 再次切换回 mock |
| 3 | 文档交付 | Task 文档更新、必要时更新 `docs/rust_frb_build.md` | Phase 1/2 完成 | 评审 spec + 文档 | 不做文档改动 |

---

## 4. 方案概要（Solution Overview）

- 设计思路：将 FRB loader `stem` 与 crate 名对齐（改为 `rust_lib_Kelivo`），通过官方 codegen 刷新生成物，维持统一入口初始化。
- 影响面：`lib/src/rust/frb_generated.dart`、`lib/main.dart`（如需传递自定义 loader）、`integration_test/*` 初始化路径、文档记录。
- 兼容性/降级：若设备不支持真实模式，可通过 `USE_RUST=false` 回落到 mock，无需加载原生库。
- 可观测性：启动日志包含加载路径；在 QA 报告中记录 context7 命令执行的时间戳与结果。
- 技术栈（Tech Stack）：
  - 架构模式：现有 Flutter MVVM-ish + Provider。
  - 状态管理：Provider。
  - 网络层：未涉及。
  - 本地存储：无变更。
  - UI 框架/样式：Flutter。
- 分层视图：
  ```
  Presentation (Flutter UI) → RustLib init (FRB) → Rust FFI (rust_lib_Kelivo)
  ```

---

## 5. 模块与调用关系（Modules & Flows）

- 模块清单
  | 模块 | 职责 | 新增/修改/复用 | 外部接口 |
  |------|------|----------------|----------|
  | `lib/src/rust/frb_generated.dart` | FRB 入口配置 | 修改 | `ExternalLibraryLoaderConfig` |
  | `lib/main.dart` | 应用启动与 init 分支 | 修改 (如需显式传入 ExternalLibrary) | `RustLib.init` / `RustLib.initMock` |
  | `integration_test/*.dart` | 启动验证 | 修改 | `RustLib.init` |
  | `docs/rust_frb_build.md` | 工程指引 | 视情况更新 | 开发流程文档 |
- 核心调用链：`main()` → `RustLib.init()` → `ExternalLibraryLoaderConfig` → iOS/macOS `dlopen` → Rust FFI 初始化。
- 状态机（如需）：无新增状态机，保持启动分支（Mock vs Real）。
- 模块按层划分：
  - Presentation：`lib/main.dart`
  - Domain：无直接改动
  - Data：`lib/src/rust/*`（生成代码）

---

## 6. 数据与模型（Data & Models）

- 领域实体：无新增实体；仅初始化逻辑。
- DTO ↔ Domain 映射：不涉及。
- 存储与迁移：不涉及数据库。
- 隐私与敏感：不涉及用户数据。
- 请求/响应模型：不涉及网络接口。

---

## 7. 合同与集成（Contracts & Integrations）

- 接口/事件清单
  | 名称 | 通信方式 | 请求 | 响应/负载 | 鉴权/幂等 | 错误语义 |
  |------|----------|------|-----------|-----------|----------|
  | `RustLib.init` | FFI | 无 | `Future<void>` | n/a | 抛出 `Invalid argument` 时加载失败 |
- 失败与降级：捕获加载失败时提示切换 mock；在文档记录排查步骤。
- 灰度与回滚：通过 `USE_RUST=false` 切换；必要时回滚到旧版本。
- 接口示例：无需新增。

---

## 8. UI 与交互（UI/UX & A11y）

- 无直接 UI 变更，关注启动稳定性即可。

---

## 9. 校验与验收（Verification & Acceptance）

- 测试层次：`fvm dart analyze`、`fvm flutter test`、`integration_test` 启动、手动真机/模拟器验证。
- 关键用例表
  | 编号 | 场景 | 前置 | 步骤 | 期望 | 验收方式 |
  |-----:|------|------|------|------|----------|
  | TC-1 | 启动（真实模式） | `USE_RUST=true` | 运行 `fvm flutter run` | 应用启动成功 | 终端日志 & 手测 |
  | TC-2 | 启动（Mock） | `USE_RUST=false` | 运行 `fvm flutter run` | 使用 mock，仍可启动 | 终端日志 |
  | TC-3 | 集成测试 | 分支 `spec/rust-lib-loader-fix` | `fvm flutter test` | 所有测试通过 | CI / context7 |

- 手测策略：遵循 `docs/rust_frb_build.md`，优先本地模拟器，无法执行则记录 “To Confirm”。

---

## 10. 性能与资源（Performance & Footprint）

- 关键路径：应用启动阶段。
- 目标：无新增性能指标；确认 `dlopen` 成功后不增加额外延迟。
- 主要优化点：避免重复加载，确保一次 init。

---

## 11. 安全与隐私（Security & Privacy）

- 无新增数据路径；确保未暴露动态库路径或敏感变量。

---

## 12. 观测与运维（Observability & Ops）

- 启动日志包含 `RustLib.init` 成功信息；如失败记录错误。
- 如通过 context7 执行命令，应保留结果以便追溯。

---

## 13. 影响评估（Impact & Change List）

- 改动清单：`lib/src/rust/frb_generated.dart`、`lib/main.dart`、`integration_test/*`、`specs/rust-lib-loader-fix/*`、可能的文档。
- 兼容性：保持与现有平台兼容；不影响 mock。
- 协作依赖：与 iOS/Android 构建脚本保持一致；告知 QA/DevOps 使用新的文档记录。

---

## 14. 迁移与回滚（Migration & Rollback）

- 数据迁移：不适用。
- 配置/版本控制：基于 branch `spec/rust-lib-loader-fix`；回滚只需恢复旧 loader 配置。
- 切换计划：完成验证后合入；失败时 revert。

---

## 15. 发布与交付（Release & Delivery）

- 分支策略：继续在 `spec/rust-lib-loader-fix` 推进；提交遵循 Git-Flow。
- CI/CD 要点：触发 Flutter 分析与测试；记录 context7 任务。
- 发布清单：包含验证日志、任务完成勾选、必要文档链接。

---

## 16. 风险与权衡（Risks & Trade-offs）

| 风险 | 影响 | 可能性 | 缓解 | 回滚触发 |
|------|------|--------|------|----------|
| Codegen 仍生成旧 stem | 启动继续失败 | 中 | 调整 `flutter_rust_bridge_codegen` 版本或手动覆盖并记录 | `dlopen` 再次报错 |
| 真机未验证 | 用户仍可能遇到问题 | 中 | 尽量使用模拟器/context7；无法执行则标注 To Confirm | QA 报错 |
| 修改生成文件冲突 | 自动化流程受阻 | 低 | 所有更改后记录 codegen 命令 | 构建失败 |

---

## 17. 代码组织与约定（Code Map & Conventions）

- 遵循现有目录结构；生成文件只通过 codegen 更新；`lib/main.dart` 使用现有 `kUseRust` 约定。

---

## 18. 评审清单（Review Checklist）

- [ ] 与 `requirements.md` 对齐。
- [ ] 每个 Phase 可运行/可测试/可回滚。
- [ ] 兼容性与风险明确。

---

## 19. 附录（Appendix）

- context7 MCP：工具命令需记录（如 `flutter_rust_bridge_codegen generate`），若不可用标记 “To Confirm” 与截止时间。
