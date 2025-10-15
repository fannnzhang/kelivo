# Markdown Media Sanitizer – Phase Log

## Phase 1 · Rust 核心逻辑实现
- ✅ **Status**: 完成
- 🛠 **Key Work**:
  - 新增 `rust/src/api/markdown_sanitizer.rs`，实现 `replace_inline_base64_images` 与 `inline_local_images_to_base64`。
  - 编写覆盖成功/失败路径的 Rust 单元测试。
- 🧪 **Commands**:
  - `cd rust && cargo test --package rust_lib_Kelivo`
- 📎 **Evidence**: 所有新测试通过；文件写入使用临时目录隔离，详细见源文件注释。

## Phase 2 · FRB 集成与 Dart 调用
- ✅ **Status**: 完成
- 🛠 **Key Work**:
  - 运行 `flutter_rust_bridge_codegen generate` 刷新桥接。
  - 更新 `lib/utils/markdown_media_sanitizer.dart` 以切换 Mock/Rust，实现错误回退与日志。
  - 新增特性开关 `lib/config/feature_flags.dart`；默认随 `USE_RUST` 开关启用 Rust。
  - 编写 `test/utils/markdown_media_sanitizer_integration_test.dart` 覆盖 Mock 与 Real 模式。
- 🧪 **Commands**:
  - `KELIVO_SANITIZER_IMAGE_DIR="$(pwd)/build/test_images" fvm flutter test test/utils/markdown_media_sanitizer_integration_test.dart`
- 📎 **Evidence**: 测试输出验证两种模式一致，错误路径回落到 Dart Mock。

## Phase 3 · 性能验证与清理
- ✅ **Status**: 完成
- 🛠 **Key Work**:
  - 新增基准 `test/benchmark/markdown_sanitizer_benchmark.dart` 与测试数据。
  - 记录性能报告 `docs/perf/markdown_sanitizer.md`。
  - Rust 模式默认启用；Mock 保留用于调试。
- 🧪 **Commands**:
  - `cargo build --release`
  - `KELIVO_SANITIZER_IMAGE_DIR="$(pwd)/build/bench_images" fvm flutter test test/benchmark/markdown_sanitizer_benchmark.dart --plain-name benchmark`
- 📎 **Evidence**: Replace 阶段性能提升约 71%，详见性能报告。

## Quality Gates
- ✅ `fvm dart format lib/config/feature_flags.dart lib/utils/markdown_media_sanitizer.dart test/utils/markdown_media_sanitizer_integration_test.dart test/benchmark/markdown_sanitizer_benchmark.dart`
- ✅ `fvm dart analyze`（存在的全局 lint 仍在，但新增代码未引入额外告警）
- ✅ `fvm flutter test test/utils/markdown_media_sanitizer_integration_test.dart`
- ✅ `KELIVO_SANITIZER_IMAGE_DIR="$(pwd)/build/bench_images" fvm flutter test test/benchmark/markdown_sanitizer_benchmark.dart --plain-name benchmark`
- ⚠️ `fvm flutter test`（完整套件仍包含默认 `widget_test.dart`，与现有应用结构不符，保持原状）
