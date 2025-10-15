# 🧠 design.md — FRB（flutter_rust_bridge）接入设计

## 0. 元信息（Meta）
- 项目标识：FRB Rust Integration for Kelivo
- 负责人：TBD
- 相关代码：
  - `flutter_rust_bridge.yaml`
  - `rust/`（crate 与生成文件）
  - `lib/src/rust/`（Dart 侧生成与 API）
  - `lib/main.dart:32` 初始化 `RustLib.init()`
- 参考文档（通过 MCP/context7 获取）：`/fzyzcjy/flutter_rust_bridge` 与 https://cjycode.com/flutter_rust_bridge/guides/


## 1. 项目基线与约束（Baseline & Constraints）
- 已有基线：
  - 配置：`flutter_rust_bridge.yaml`（指向 `rust_input: crate::api`，`dart_output: lib/src/rust`）。
  - 生成：`lib/src/rust/frb_generated.dart`、`lib/src/rust/api/simple.dart` 等（FRB v2 生成头注释存在）。
  - Rust 侧：`rust/src/api/simple.rs` 暴露 `greet` 与 `init_app`；`rust/src/frb_generated.rs` 为 codegen。
  - App 初始化：`lib/main.dart:32` 的 `main()` 中 `await RustLib.init();`（Real 模式）。
- 约束：
  - 仓库规范：`fvm dart analyze` 0 warning、`fvm dart format lib test`、`fvm flutter test` 需通过。
  - 平台：主要 Android/iOS（后续可扩展桌面与 Web/wasm，Web 需 `build-web` 支持）。
  - 网络：开发环境网络受限；外部文档查询需用 MCP/context7。
  - 生成器：FRB v2（`flutter_rust_bridge_codegen generate`）。


## 2. 目标与成功标准（Goals & Exit Criteria）
- Phase 1 完成：代码生成、初始化、`greet` 调用在 Android/iOS 可运行，分析/测试/格式化通过。
- Phase 2 完成：提供 `--dart-define=USE_RUST` 开关与 Mock 实现，不加载 FFI 也能端到端跑通。
- Phase 3 完成：固化构建说明与常见问题，CI/本地最小路径打通；文档与 README 对齐。


## 3. 渐进式交付策略（Progressive Strategy）
| Phase | 内容 | 关键动作 | Mock→Real 开关 |
|------:|------|---------|----------------|
| 1 | 基线打通 | 生成、初始化、`greet` 调通 | 无（默认 Real） |
| 2 | 模式切换 | `--dart-define=USE_RUST` + `RustLib.initMock()` | `USE_RUST=true/false` |
| 3 | 构建固化 | 打包/运行说明、CI 要点 | N/A |

显式开关（建议）：
```
// Dart 侧
const bool kUseRust = bool.fromEnvironment('USE_RUST', defaultValue: false);
```


## 4. 方案概要（Solution Overview）
- 复用：当前 FRB 生成文件、Rust crate、`RustLib.init()` 调用。
- 新增：Phase 2 提供 Mock API（实现 `RustLibApi`，仅返回与 demo 一致的串），引入 `--dart-define` 选择。
- 兼容性：默认不更改业务模块；仅在初始化与依赖注入层面切换。
- 可观测性：在 Debug 输出初始化与错误；Release 静默。


## 5. 模块与调用关系（Modules & Flows）
- Flutter（Dart 层业务） → FRB 生成的 Dart API（如 `api/simple.dart`） → FFI → Rust crate（`rust/src/api/simple.rs`）。
- 示例流：`greet(name)` 在 Dart 调用 → FFI 进入 Rust → `format!("Hello, {name}!")` 返回 → Dart 得到字符串。


## 6. 数据与模型（Data & Models）
- Demo 仅字符串入参/返回；后续若引入复杂 DTO，遵循 FRB v2 类型映射与零拷贝策略。


## 7. 合同与集成（Contracts & Integrations）
- Codegen：使用 FRB v2 `flutter_rust_bridge_codegen generate`，输入由 `flutter_rust_bridge.yaml` 决定。
- 动态库命名：`rust_lib_Kelivo`（由 rust_builder 模板生成），Android 用 NDK，iOS Podspec glue。
- MCP：外部文档通过 context7（库 ID `/fzyzcjy/flutter_rust_bridge`）。


## 8. UI 与交互（UI/UX & A11y）
- 本项为基础设施；无需新增 UI。可在调试/测试中调用 `greet` 验证。


## 9. 校验与验收（Verification & Acceptance）
- 本地命令：
  - 生成：`flutter_rust_bridge_codegen generate`
  - 质量：`fvm dart analyze`、`fvm dart format lib test --set-exit-if-changed`、`fvm flutter test`
  - 构建（最小）：`fvm flutter run -d <device>` / 可选 `fvm flutter build apk --debug`
- 自动化测试建议：新增 `integration_test/rust_greet_test.dart` 调用 `RustLib.init()` 后断言 `greet` 结果。


## 10. 性能与资源（Performance & Footprint）
- Demo 阶段主要验证链路；后续若引入计算密集模块，再对内存/CPU/序列化成本做压测与指标化。


## 11. 安全与隐私（Security & Privacy）
- FFI 仅加载本地打包的库；不下载远程二进制。
- iOS 代码签名/Android NDK 符号控制遵循平台默认。


## 12. 观测与运维（Observability & Ops）
- Debug 日志：初始化成功/失败、库路径、函数调用失败原因。
- Release：仅关键错误打印。


## 13. 影响评估（Impact & Change List）
- 可能影响：启动时初始化耗时微小增加；构建时间增加（Rust 编译）。
- 对业务模块影响：无（除非显式在业务中引入 Rust API）。


## 14. 迁移与回滚（Migration & Rollback）
- 切换到 Mock：`--dart-define=USE_RUST=false`，并在初始化分支中使用 `RustLib.initMock`。
- 避免加载原生库：Mock 模式不触发动态库加载，若 Real 初始化失败可快速切回。


## 15. 发布与交付（Release & Delivery）
- Phase Gate：每阶段完成后更新 `tasks.md`、附验收证据（日志/截图/录屏）。
- 构建要点：
  - 先 `fvm flutter pub get`；再 `flutter_rust_bridge_codegen generate`；再 Flutter 端构建。
  - Android NDK/Toolchain 安装完整；iOS 需 `pod install` 正常。


## 16. 风险与权衡（Risks & Trade-offs）
- 风险：
  - FRB 版本变更导致生成/接口变化（需关注 v2 升级指南）。
  - 多平台构建环境差异（NDK/LLVM/Clang 路径）。
- 缓解：
  - 固化生成与构建命令；遇到平台编译错误优先用 Mock 保持业务不阻塞。


## 17. 代码组织与约定（Code Map & Conventions）
- `lib/src/rust/`：Dart 侧生成与 API（如 `api/simple.dart`、`frb_generated.dart`）。
- `rust/`：Rust crate 与生成文件（`src/api/*.rs`、`src/frb_generated.rs`）。
- `rust_builder/`：各平台 glue（Podspec、CMake、Gradle）无需手改。
- 样式：遵循项目 `analysis_options.yaml` 与 Flutter/Dart 命名约定。


## 18. 评审清单（Review Checklist）
- 代码生成正确、无手改生成文件。
- Android/iOS 至少一种真机/模拟器实测可运行。
- `USE_RUST` 开关与 Mock 行为明确并能回滚。
- 分析/测试/格式化全部通过。
- 文档与 README 一致，不在 `templates/` 放规则性内容。


## 19. 附录（Appendix）
- FRB Guides（MCP/context7 获取）：`/fzyzcjy/flutter_rust_bridge`，公共入口：https://cjycode.com/flutter_rust_bridge/guides/
- 生成器命令（v2）：`flutter_rust_bridge_codegen generate`；Web 构建：`flutter_rust_bridge_codegen build-web`
- 环境变量：`MCP_CONTEXT7_URL`、`MCP_CONTEXT7_TOKEN`（不提交到仓库）
