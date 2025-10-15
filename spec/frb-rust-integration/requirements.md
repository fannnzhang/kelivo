# 📄 requirements.md — FRB（flutter_rust_bridge）接入与文档更新

## 1. Introduction（需求背景）
Kelivo 项目已引入 flutter_rust_bridge（FRB）以支持 Rust 能力（已存在目录 `rust/`、生成产物 `lib/src/rust/*`、配置 `flutter_rust_bridge.yaml`，以及在 `lib/main.dart` 中初始化 `RustLib.init()`）。本需求旨在：
- 为 FRB 接入补齐规范化的需求与文档；
- 明确 Mock→Real 渐进式策略与开关；
- 制定多平台（Android/iOS/桌面/可选 Web）构建与验收要求；
- 约束代码生成、依赖与安全策略；
- 与现有 README、代码与仓库约定保持一致。

参考资料（通过 MCP/context7 获取与核对）：
- FRB 官方文档（Guides/Manual/Quickstart）：https://cjycode.com/flutter_rust_bridge/guides/
- Context7 Library: /fzyzcjy/flutter_rust_bridge（使用 MCP context7 工具获取细节与命令）


## 2. 需求描述（Requirements）
采用 EARS 语法撰写（WHEN/THEN/SHALL）：
- WHEN 开发者在本地/CI 执行代码生成 THEN 系统 SHALL 依据 `flutter_rust_bridge.yaml` 生成 `lib/src/rust` 代码 SO THAT Dart↔Rust 绑定保持最新。
- WHEN 应用启动 THEN 系统 SHALL 初始化 FRB（默认 Real，后续引入 Mock/Real 可切换） SO THAT Rust FFI 在运行期可用。
- WHEN Dart 调用 `greet(name)` THEN 系统 SHALL 返回 `Hello, <name>!`（来自 Rust demo） SO THAT 证明绑定正常工作。
- WHEN 在 Android/iOS 本地构建 THEN 系统 SHALL 正确链接 `rust_lib_Kelivo` 并通过运行期加载 SO THAT 移动端实现端到端可运行。
- WHEN 执行 `fvm dart analyze`/`fvm flutter test`/`fvm dart format` THEN 系统 SHALL 通过检查 SO THAT 保持仓库质量一致。
- WHEN 切换 `--dart-define=USE_RUST=<true|false>`（新增） THEN 系统 SHALL 在 Mock 与 Real 实现之间切换（Mock 不加载 FFI） SO THAT 便于本地开发与回滚。


## 3. 分阶段开发策略（Phased Development Strategy）
| Phase | 标题 | 目标 | 可运行/可回滚 | 开关/切换 |
|------:|------|------|--------------|-----------|
| 1 | FRB 基线验证（Hello Rust） | 保证代码生成、初始化、调用 `greet` 端到端通过 | 可独立运行；失败时不影响其它功能 | 暂无（默认 Real） |
| 2 | Mock/Real 可切换 | 新增 `--dart-define=USE_RUST` 与 DI 选择；提供 Mock API | 任一模式可独立运行、快速回滚 | `USE_RUST=true/false` |
| 3 | 构建与文档固化 | 完成多平台最小构建路径、补齐开发文档与 CI 要点 | 可独立验证与回滚 | N/A |


## 4. Requirements（详细需求）

### Phase 1: FRB 基线验证（Hello Rust）
- User Story：
  - 作为开发者，我希望通过最小示例验证 FRB 绑定能从 Dart 调到 Rust 并返回期望结果，以便确认链路无误。
- Acceptance Criteria（EARS）：
  - WHEN 运行 `flutter_rust_bridge_codegen generate` THEN 系统 SHALL 在 `lib/src/rust` 下生成/更新桥接代码。
  - WHEN 启动应用 THEN 系统 SHALL 执行 `RustLib.init()` 完成初始化。
  - WHEN 调用 `greet('World')` THEN 系统 SHALL 返回 `Hello, World!`。
  - WHEN 在 Android/iOS 本地构建运行 THEN 系统 SHALL 正常加载 `rust_lib_Kelivo` 且无运行时链接错误。

### Phase 2: Mock/Real 可切换
- User Story：
  - 作为开发者，我希望在不加载 FFI 的情况下进行业务快速验证（Mock），并在需要时再切换到真实 Rust 实现（Real）。
- Acceptance Criteria（EARS）：
  - WHEN 以 `--dart-define=USE_RUST=false` 启动 THEN 系统 SHALL 使用 `RustLib.initMock(api: ...)`，且不加载任何原生库。
  - WHEN 以 `--dart-define=USE_RUST=true` 启动 THEN 系统 SHALL 使用 `RustLib.init()` 并加载 FFI 库。
  - WHEN 在两种模式下调用 `greet` THEN 系统 SHALL 分别从 Mock/Real 返回一致结果（字符串相同）。

### Phase 3: 构建与文档固化
- User Story：
  - 作为维护者，我希望完善多平台最小构建路径、CI 要点、常见问题与开发文档，降低后续演进与协作成本。
- Acceptance Criteria（EARS）：
- WHEN 执行 `fvm dart analyze`、`fvm flutter test`、`fvm dart format lib test` THEN 系统 SHALL 通过检查。
  - WHEN 按文档步骤运行 Android/iOS Debug 构建 THEN 系统 SHALL 成功产出并运行示例调用。
  - WHEN 检查文档（spec 与 README/Docs 对齐） THEN 系统 SHALL 保持一致、不冲突，并提供 MCP/context7 的引用与变量说明。


## 5. Non-functional & Cross-cutting（非功能与横切）
- 技术架构与代码规范：
  - 严格遵循仓库约定（Dart 2 空格缩进、`flutter_lints`、snake_case 文件命名）。
  - 代码生成、桥接文件存放在 `lib/src/rust/`；Rust crate 在 `rust/`；构建胶水在 `rust_builder/`。
  - 不在模板 `templates/` 内放入用法指引；本规范全部集中在 `spec/` 与现有 README/docs。
- 错误处理与用户体验：
  - 初始化失败需日志可见；在 Mock 模式下不因无原生库而崩溃。
  - 运行期 FFI 错误（库找不到、符号缺失）需在控制台清晰提示并可回退到 Mock。
- 性能与安全：
  - 仅桥接必要 API；避免不必要的内存复制；遵循 FRB v2 零拷贝默认策略。
  - FFI 库签名与打包遵循平台规范（iOS Pod、Android NDK 链接等）。
- 可观测与运维：
  - 在 Debug 构建输出关键日志（初始化、调用、错误），Release 最小化噪音。
- MCP 使用与密钥：
  - 外部文档检索使用 MCP context7 工具；通过环境变量 `MCP_CONTEXT7_URL`、`MCP_CONTEXT7_TOKEN` 配置；不提交任何密钥。
  - 若 MCP 不可用，标记为 “To Confirm”，采用本地 Mock-first 策略，避免硬编码。
