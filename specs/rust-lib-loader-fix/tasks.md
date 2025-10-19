# 🛠 tasks.md — Implementation Plan（rust_lib_Kelivo Loader 修复）

> 📌 元信息（Metadata）：
> - Spec Branch: `spec/rust-lib-loader-fix`
> - 所有提交必须推送到该分支，不得直接提交到主分支。
> - 每个 Phase 执行完成的验收条件之一：对应修改已按 `Git-Flow.md` 规范提交。

---

## 分阶段开发策略（Phases Overview）

- Phase 1: Loader 修正（统一 FRB `stem` 命名）
- Phase 2: 生成与验证（codegen + 分析测试 + 启动验证）
- Phase 3: 文档与交付（记录流程、更新 spec 状态）
- Phase 4: 预留（如需追加平台验证）

---

## Phase 1: Loader 修正

- [x] 1. 更新 FRB 默认 stem
  - Summary: 调整 `ExternalLibraryLoaderConfig` 使其与 crate 名称 `rust_lib_Kelivo` 对齐。
  - Files:
    - `lib/src/rust/frb_generated.dart`
  - Changes:
    - 将 `stem: 'rust_lib_kelivo'` 改为 `stem: 'rust_lib_Kelivo'`。
    - 确认无需手动修改其他生成字段。
  - Requirements: `R1`
  - Acceptance:
    - `git diff` 显示新 stem，与 Rust crate 命名一致。
    - 再次运行 `USE_RUST=true` 时不再引用小写库名（可先局部验证或在 Phase 2 启动）。

---

## Phase 2: 生成与验证

- [x] 2. 重新生成 FRB 绑定
  - Summary: 按规范运行 `flutter_rust_bridge_codegen generate`，同步所有生成文件。
  - Files:
    - `lib/src/rust/*`
  - Changes:
    - 使用 MCP context7 工具链或本地命令执行 `flutter_rust_bridge_codegen generate`。
    - 确认 `kDefaultExternalLibraryLoaderConfig` 中 stem 保持大写版本。
  - Requirements: `R2`
  - Acceptance:
    - 记录命令输出（本地或 context7）；若 context7 不可用，标注 “To Confirm” 与截止日期。
    - `git status` 仅显示预期的生成文件变更。

- [ ] 2.1 Flutter 静态检查
  - Summary: 执行 `fvm dart analyze` 与 `fvm flutter test`，验证无回归。
  - Files:
    - `lib/**`, `test/**`, `integration_test/**`
  - Changes:
    - 无代码变更；收集测试结果。
  - Requirements: `R3`
  - Acceptance:
    - 命令全部通过；若环境不足，记录 “To Confirm” 与计划。
    - Notes: 2025-02-14 `fvm dart analyze` 仍因既有依赖缺失（`package:hex`、`data_sync` 参数变更）报错，待后续修复。

- [ ] 2.2 实机/模拟器启动验证
  - Summary: 使用 `fvm flutter run --dart-define=USE_RUST=true`（或 context7 CI 任务）验证真实模式启动。
  - Files:
    - `lib/main.dart`（观察日志）
  - Changes:
    - 无代码变更；获取启动日志确认 `RustLib.init` 成功。
  - Requirements: `R3`
  - Acceptance:
    - 启动成功并无 `dlopen` 错误；若无法执行，记录 “To Confirm” 并指派负责人/截止日期。
    - Notes: 2025-02-14 本地 `fvm flutter test` 仍缺失 `rust_lib_Kelivo.framework`，需通过构建脚本/真实打包确认。

---

## Phase 3: 文档与交付

- [ ] 3. 更新文档与任务记录
  - Summary: 根据执行结果更新 `docs/rust_frb_build.md`（若需）与本 spec 下 `tasks.md` 勾选状态。
  - Files:
    - `docs/rust_frb_build.md`（如需更新）
    - `specs/rust-lib-loader-fix/tasks.md`
  - Changes:
    - 加入验证记录、context7 使用说明或待确认项。
  - Requirements: `R4`
  - Acceptance:
    - 文档与 spec 一致；没有新增未授权依赖。

---

## Phase 4: 预留

- [ ] 4. 平台扩展验证（可选）
  - Summary: 若后续需要，在 Android/Linux/Windows 上验证加载结果。
  - Files:
    - `specs/rust-lib-loader-fix/tasks.md`（记录状态）
  - Changes:
    - 记录执行情况或备注待确认。
  - Requirements: `R1`, `R3`
  - Acceptance:
    - 验证完成或明确说明原因与计划。

---

## 贯穿所有阶段的任务（Cross-phase Tasks）

- [ ] X. MCP/context7 记录
  - Summary: 所有关键命令（codegen、测试、构建）优先通过 MCP context7 执行或同步结果。
  - Files:
    - `specs/rust-lib-loader-fix/tasks.md`（记录时间与状态）
  - Changes:
    - 无代码变更；补齐执行日志链接或 “To Confirm”。
  - Requirements: `R2`, `R3`, `R4`
  - Acceptance:
    - 若 context7 不可用，明确说明原因与后续计划；否则提供执行记录引用。
