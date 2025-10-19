# 🛠 tasks.md — Implementation Plan（落地实施）

> 📌 元信息（Metadata）：
> - Spec Branch: `specs/flutter-build-fix`
> - 所有提交必须推送到该分支，不得直接提交到主分支。
> - 每个 Phase 执行完成的**验收条件之一**：对应修改已按 [Git-Flow.md](../Git-Flow.md) 规范提交。

---

## 分阶段开发策略（Phases Overview）

- Phase 1: Align mock API sources（同步 imports、签名、模型默认值）
- Phase 2: Validate build health（运行 analyzer/test，记录结果）
- Phase 3: N/A 本次无需
- Phase 4: N/A 本次无需

---

## Phase 1: Align mock API sources

- [ ] 1. Update mock imports and dependencies
  - Summary: 调整 `lib/src/rust/mock_api.dart` 的包路径与依赖，加入 `llm_types.dart` 与 `flutter_rust_bridge_for_generated.dart`。
  - Files:
    - `lib/src/rust/mock_api.dart`
  - Changes:
    - 将 `package:kelivo/...` 更换为 `package:Kelivo/...`。
    - 新增 `package:Kelivo/src/rust/api/llm_types.dart` 及 `flutter_rust_bridge_for_generated.dart` 导入。
  - Requirements: `R1`
  - Acceptance:
    - `flutter analyze lib/src/rust/mock_api.dart` 不再出现 URI 或类型实现错误。
    - 提交本次修改的代码

- [ ] 1.1 Sync RustLibApi method contracts
  - Summary: 更新 mock 方法签名与返回值，移除不存在的方法，提供符合新模型的默认数据。
  - Files:
    - `lib/src/rust/mock_api.dart`
  - Changes:
    - 匹配 `RustLibApi` 所有方法签名（包含 `PlatformInt64` 参数）。
    - 返回值使用新结构（如 `FrbChatResponse`、`FrbDbInfo`），删除 `crateApiDbDbUpsertToolCall` 等陈旧实现。
    - 为 `FrbChatResponse.usage`、`FrbDbInfo` 等提供合理 mock 默认值。
  - Requirements: `R2`
  - Acceptance:
    - `flutter analyze` 对 `lib/src/rust/mock_api.dart` 无 `override_on_non_overriding_member` 或类型错误。
    - 提交本次修改的代码

---

## Phase 2: Validate build health

- [ ] 2. Run analyzer and tests
  - Summary: 运行 Flutter 静态检查与单元测试，确认构建恢复。
  - Files:
    - `lib/main.dart`
    - `lib/src/rust/mock_api.dart`
  - Changes:
    - 无代码改动，仅执行并记录命令输出要点。
  - Requirements: `R3`
  - Acceptance:
    - `flutter analyze` 完成且无 error。
    - `flutter test --concurrency 1` 通过（允许既有 warning）。
    - 在工作日志或备注中记录命令与结果。
    - 提交本次修改的代码

---

## Phase 3: N/A 本次无需

- 无任务。

---

## Phase 4: N/A 本次无需

- 无任务。

---

## 贯穿所有阶段的任务（Cross-phase Tasks）

- [ ] X. Alerter progress notifications
  - Summary: 每阶段开始/结束使用 `alerter` 发送进度通知，遵循 `specify/Alerter-Usage.md`。
  - Files:
    - 无（命令执行）
  - Changes:
    - 在关键节点执行 `alerter -title "Codex" -message "..." > /dev/null 2>&1 &`。
  - Requirements: `R1`, `R2`, `R3`
  - Acceptance:
    - 所有阶段遵循通知规范；若 `alerter` 不可用则记录。
    - 提交本次修改的代码（如无代码改动，更新任务备注）。
