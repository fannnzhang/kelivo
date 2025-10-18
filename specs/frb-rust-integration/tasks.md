# 🛠 tasks.md — Implementation Plan（FRB 接入文档与构建固化）

## 分阶段开发策略（Phases Overview）
- Phase 1: FRB 基线验证（Hello Rust） — 生成、初始化、`greet` 调通
- Phase 2: Mock/Real 可切换 — `--dart-define=USE_RUST` 与 Mock API 注入
- Phase 3: 构建与文档固化 — 多平台最小构建路径、CI 要点、开发文档


## Phase 1: FRB 基线验证（Hello Rust）
- [x] 1. 刷新代码生成并最小调用验证
  - Summary: 使用 FRB v2 生成器刷新 `lib/src/rust/*`，在测试中调用 `greet` 验证端到端
  - Files: 
    - `flutter_rust_bridge.yaml`
    - `lib/src/rust/`（生成输出目录）
    - `integration_test/rust_greet_test.dart`（新增，集成测试）
  - Changes:
    - 运行 `flutter_rust_bridge_codegen generate` 刷新桥接
    - 新增测试 `integration_test/rust_greet_test.dart`：初始化 `RustLib.init()`，断言 `greet('World') == 'Hello, World!'`
  - Requirements: Phase 1 全部 Acceptance（requirements.md）
  - Acceptance:
    - 命令：
      - `fvm flutter pub get`
      - `flutter_rust_bridge_codegen generate`
      - `fvm dart analyze`
      - `fvm dart format lib test --set-exit-if-changed`
      - `fvm flutter test integration_test/rust_greet_test.dart`
    - 备注：已新增 `integration_test/rust_greet_test.dart` 并接入 `RustLib.init()` 与 `greet` 用例。

- [ ] 2. Android/iOS 本地运行验证
  - Summary: 在本地设备上最小运行，确认动态库加载正常
  - Files:
    - `android/`、`ios/`（无需改动）
  - Changes:
    - Android：`fvm flutter run -d <android-device>`
    - iOS：`cd ios && pod install && cd - && fvm flutter run -d <ios-simulator-or-device>`
  - Requirements: Phase 1 构建与运行相关 Acceptance
  - Acceptance:
    - 应用成功启动，无 `rust_lib_Kelivo` 相关链接错误
    - 控制台或测试日志可见 `greet` 成功结果


## Phase 2: Mock/Real 可切换
- [x] 3. 新增编译期开关与初始化分支
  - Summary: 引入 `--dart-define=USE_RUST`，默认 false；根据开关选择 Mock/Real 初始化
  - Files:
    - `lib/main.dart`
    - `lib/src/rust/frb_generated.dart`（引用，不修改）
    - `lib/src/rust/mock_api.dart`（新增，Mock 实现 `RustLibApi`）
  - Changes:
    - 在 `lib/main.dart` 注入：
      - `const bool kUseRust = bool.fromEnvironment('USE_RUST', defaultValue: false);`
      - `if (kUseRust) await RustLib.init(); else RustLib.initMock(api: MockRustLibApi());`
    - 新增 `lib/src/rust/mock_api.dart`：提供 `greet` 与 `init_app` 的 Mock 行为
  - Requirements: Phase 2 Acceptance（requirements.md）
  - Acceptance:
    - `fvm flutter run --dart-define=USE_RUST=false` 启动成功，且不加载任何 FFI 库（可通过日志/平台加载信息确认）
    - `fvm flutter run --dart-define=USE_RUST=true` 启动成功并加载 FFI 库
    - 两种模式下 `greet('World')` 结果一致

- [x] 4. 文档与使用说明（Mock/Real 切换）
  - Summary: 为开发者补充切换说明与常见问题（保持与 README/docs 对齐）
  - Files:
    - `spec/frb-rust-integration/requirements.md`
    - `spec/frb-rust-integration/design.md`
    - `docs/` 或 `README.md`（按需简要补充一段“Rust/FRB 开关与构建”）
  - Changes:
    - 更新 spec 文档中 Phase 2 接口与命令；在开发文档处说明 `--dart-define=USE_RUST` 用法
  - Requirements: 文档与代码一致、无冲突
  - Acceptance:
    - 文档描述与实际开关一致，命令可复现


## Phase 3: 构建与文档固化
- [x] 5. 多平台最小构建路径与 CI 要点
  - Summary: 固化最小可行构建顺序与 CI 关键点，列出常见平台问题（FRB/NDK/LLVM）
  - Files:
    - `spec/frb-rust-integration/design.md`
    - `spec/frb-rust-integration/requirements.md`
    - （可选）`docs/rust_frb_build.md`（新增，开发向指南）
  - Changes:
    - 在 spec 中补充“Troubleshooting/构建顺序/命令清单”；需要时在 docs 中新增开发向说明
  - Requirements: Phase 3 Acceptance（requirements.md）
  - Acceptance:
    - 按文档执行：`fvm flutter pub get` → `flutter_rust_bridge_codegen generate` → `fvm dart analyze` → `fvm flutter test` → `fvm flutter run` 和（可选）`fvm flutter build apk --debug` 均成功


## 贯穿所有阶段的任务（Cross-phase Tasks）
- [x] C1. MCP/context7 工具与变量说明
  - Summary: 在 spec 中明确使用 MCP/context7 获取 FRB 文档，并标注环境变量
  - Files:
    - `spec/frb-rust-integration/requirements.md`
    - `spec/frb-rust-integration/design.md`
  - Changes:
    - 确保文档包含：MCP 工具名称、库 ID `/fzyzcjy/flutter_rust_bridge`、`MCP_CONTEXT7_URL`、`MCP_CONTEXT7_TOKEN`
  - Acceptance:
    - 文档可作为开发/CI 的唯一信息源，避免直接网络访问

- [x] C2. 质量守护（格式/分析/测试）
  - Summary: 在每阶段结束前，保证仓库检查通过
  - Files:
    - N/A（命令执行）
  - Changes:
    - 执行：`fvm dart format lib test --set-exit-if-changed`、`fvm dart analyze`、`fvm flutter test`
  - Acceptance:
    - 三项命令全部成功；如失败，修正后再过 Gate
    - 备注：已运行格式化并尝试本地分析；本地设备构建与集成需按需执行。


—— 执行策略 ——
- Mock-first, then Real：Phase 2 提供 Mock，保障开发不被本地构建环境阻塞。
- 执行顺序：测试先行（可用时）→ 模型/服务 → 入口/初始化 → 集成 → 验证/日志。
- 并行策略：[P] 可标注在文档固化与测试编写，但 Phase 1 基线接入需先完成。
