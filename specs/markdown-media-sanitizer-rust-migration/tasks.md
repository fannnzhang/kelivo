# 🛠 tasks.md — MarkdownMediaSanitizer 迁移 Rust 实施计划

## 分阶段开发策略（Phases Overview）

- Phase 1: Rust 核心逻辑实现（在 Rust 端实现 Markdown 图片处理的核心功能并编写单元测试）
- Phase 2: FRB 集成与 Dart 调用（通过 FRB 暴露 Rust 函数，并在 Dart 中切换到 Rust 实现）
- Phase 3: 性能验证与清理（对比性能并移除旧实现，完成切换）

---

## Phase 1: Rust 核心逻辑实现

- [x] 1. 创建 Rust 模块和文件结构
  - Summary: 在 `rust/src/api/` 下新增 `markdown_sanitizer.rs` 模块，并在 `mod.rs` 中导出供其他模块使用。
  - Files:
    - `rust/src/api/markdown_sanitizer.rs`
    - `rust/src/api/mod.rs`
  - Changes:
    - 定义 `markdown_sanitizer` 模块骨架并确保按现有项目约定组织代码。
    - 在 `mod.rs` 中 `pub mod markdown_sanitizer;`，与现有模块注册方式保持一致。
  - Requirements: R1, R2
  - Acceptance:
    - `cargo check` 在 `rust/` 目录执行成功，确认模块被正确识别。
    - Notes: 2025-10-15 `cargo test --package rust_lib_Kelivo` ✅

- [x] 1.1 实现 `replace_inline_base64_images`
  - Summary: 在 Rust 端实现 Base64 图片落盘逻辑，返回更新后的 Markdown。
  - Files:
    - `rust/src/api/markdown_sanitizer.rs`
  - Changes:
    - 添加 `pub fn replace_inline_base64_images(markdown: String) -> Result<String, String>`.
    - 使用 `regex` 匹配 Markdown 内联 Base64 图片语法并提取内容。
    - 使用 `base64` 解码图片数据，将文件写入项目配置的 `images/` 目录。
    - 通过 `uuid::Uuid::new_v5` 基于内容生成确定性文件名，返回替换文件路径的 Markdown。
  - Requirements: R1
  - Acceptance:
    - 针对该函数的 Rust 单元测试覆盖成功路径和错误路径，并通过 `cargo test --package rust_lib_Kelivo`。
    - Notes: 单元测试位于 `rust/src/api/markdown_sanitizer.rs`，伪造目录使用环境变量隔离。

- [x] 1.2 实现 `inline_local_images_to_base64`
  - Summary: 将 Markdown 中的本地图片文件替换为 Base64 数据 URI。
  - Files:
    - `rust/src/api/markdown_sanitizer.rs`
  - Changes:
    - 添加 `pub fn inline_local_images_to_base64(markdown: String) -> Result<String, String>`.
    - 使用 `regex` 匹配 Markdown 内的本地图片路径并读取文件为二进制。
    - 使用 `base64` 编码文件内容，拼接为 `data:` URI 并替换原路径。
    - 针对文件不存在或不可读的情况返回 `Err` 或保留原文，遵循需求约束。
  - Requirements: R2
  - Acceptance:
    - `cargo test --package rust_lib_Kelivo` 包含正向和异常用例并全部通过。
    - Notes: 覆盖本地路径缺失、远程 URL 旁路等场景。

- [x] 1.3 编写 Rust 单元测试
  - Summary: 为 Rust 模块新增全面的单元测试，覆盖正常流程与失败场景。
  - Files:
    - `rust/src/api/markdown_sanitizer.rs`
  - Changes:
    - 在 `#[cfg(test)]` 模块中添加针对 Base64 替换与本地文件内联的测试。
    - 使用临时目录或 `tempfile` 等工具隔离文件写入，确保测试可重复执行。
    - 覆盖空输入、无匹配、错误数据等边界条件。
  - Requirements: R1, R2
  - Acceptance:
    - 在 `rust/` 目录执行 `cargo test --package rust_lib_Kelivo`，所有新测试通过并生成测试证据。
    - Notes: 见 `docs/progress/markdown_media_sanitizer.md` Phase 1 日志。

---

## Phase 2: FRB 集成与 Dart 调用

- [x] 2. 暴露 Rust 函数给 FRB
  - Summary: 更新 FRB 暴露层，确保新函数可被自动生成功能识别。
  - Files:
    - `rust/src/api/mod.rs`
    - `rust/src/api/api.rs`（若项目使用该命名）
  - Changes:
    - 在 API 汇总文件中通过 `pub use` 暴露 `replace_inline_base64_images` 与 `inline_local_images_to_base64`。
    - 根据项目脚本运行 `flutter_rust_bridge_codegen`（通过 MCP/context7 工具链），生成最新绑定。
    - 校验生成文件被添加到 `.gitignore` 或版本控制策略中。
  - Requirements: R4
  - Acceptance:
    - 运行 FRB 生成脚本后，`lib/src/rust/api/api.dart`（或项目当前生成路径）新增对应函数签名，`fvm dart analyze` 通过。
    - Notes: `flutter_rust_bridge_codegen generate` @2025-10-15；生成文件位于 `lib/src/rust/api/markdown_sanitizer.dart`。

- [x] 2.1 更新 Dart 端实现
  - Summary: 调整 `MarkdownMediaSanitizer` 使其委托给 Rust，并保留 Mock/Real 开关。
  - Files:
    - `lib/utils/markdown_media_sanitizer.dart`
    - `lib/config/feature_flags.dart`（若不存在则新建）
  - Changes:
    - 引入配置或 DI 开关（如 `MarkdownSanitizerMode` 枚举）以在 Mock（旧 Dart 实现）与 Real（Rust 调用）之间切换。
    - 在 `replaceInlineBase64Images` 与 `inlineLocalImagesToBase64` 中调用生成的 FRB API，并处理 `Result` 错误分支，记录日志。
    - 保留旧实现作为 Mock 分支，确保 Phase 2 完成时可快速回滚。
  - Requirements: R3, R5
  - Acceptance:
    - 启用 Mock 模式运行现有单元测试确保无回归，再启用 Real 模式运行 `fvm flutter test` 与手动 Markdown 场景测试，行为与迁移前一致。
    - Notes: `test/utils/markdown_media_sanitizer_integration_test.dart` 覆盖 Mock/Real；命令详见进度日志。

- [x] 2.2 编写 Dart 集成测试
  - Summary: 新增集成测试验证 Dart 调用 Rust 时的端到端行为。
  - Files:
    - `test/utils/markdown_media_sanitizer_integration_test.dart`
    - `test/testdata/markdown/`（若需要测试数据则新建）
  - Changes:
    - 在测试中构造包含 Base64 图片与本地图片路径的 Markdown，分别验证 Mock 与 Real 模式输出。
    - 使用 FRB 生成的 API，确保错误路径返回 `Result.err()` 时 Dart 层能捕获并处理。
    - 将测试纳入项目现有测试套件并记录运行命令。
  - Requirements: R3, R4
  - Acceptance:
    - `fvm flutter test test/utils/markdown_media_sanitizer_integration_test.dart` 在 Mock 和 Real 模式下均通过，日志无异常。
    - Notes: 环境变量 `KELIVO_SANITIZER_IMAGE_DIR` 指向 `build/test_images` 以隔离输出。

---

## Phase 3: 性能验证与清理

- [x] 3. 构建性能基准测试
  - Summary: 编写基准测试评估 Markdown 图片处理在 Mock 与 Real 模式的性能差异。
  - Files:
    - `test/benchmark/markdown_sanitizer_benchmark.dart`
    - `test/testdata/benchmark/`（生成大样本数据）
  - Changes:
    - 使用项目基准测试框架在同一数据集上分别调用 Mock（Dart）和 Real（Rust）实现。
    - 记录运行时间与内存占用，生成对比报告并存档至 `docs/perf/markdown_sanitizer.md`。
    - 在报告中标注环境、命令以及 `fvm flutter test --plain-name benchmark` 等执行方式。
  - Requirements: R3（兼容验证），非功能性性能目标
  - Acceptance:
    - 基准结果显示 Real 模式较 Mock 模式性能提升 ≥30%，报告同步至仓库并在 spec 进展日志记录。
    - Notes: Replace 阶段提升 ~71%；详见 `docs/perf/markdown_sanitizer.md`。

- [x] 3.1 移除旧实现并完成清理
  - Summary: 在性能验证完成后删除不再需要的 Dart 旧逻辑并整理依赖。
  - Files:
    - `lib/utils/markdown_media_sanitizer.dart`
    - `lib/config/feature_flags.dart`
    - `test/utils/markdown_media_sanitizer_integration_test.dart`
  - Changes:
    - 将 Mock 模式切换为调用 Rust（或标记为仅调试使用），移除不再使用的私有函数、常量与测试数据。
    - 更新文档与注释，明确 Rust 为默认实现，并确保 Feature Flag 默认 Real。
    - 运行 `fvm dart format lib test` 与 `fvm dart analyze`，确认无遗留警告。
  - Requirements: R5
  - Acceptance:
    - 所有相关测试（`cargo test`、`fvm flutter test`）和静态检查通过，仓库无残留旧逻辑，Mock 开关默认关闭。
    - Notes: Rust 模式默认启用；Mock 分支仅用于调试 fallback。

---

## 贯穿所有阶段的任务（Cross-phase Tasks）

- [x] X. 渠道同步与进度日志
  - Summary: 保持 spec 与实现同步，记录 Phase Gate 进展和证据。
  - Files:
    - `spec/markdown-media-sanitizer-rust-migration/tasks.md`
    - `spec/markdown-media-sanitizer-rust-migration/design.md`
    - `docs/progress/markdown_media_sanitizer.md`
  - Changes:
    - 在每个 Phase 结束后更新 tasks.md 勾选状态与备注，附加日志/截图链接。
    - 将性能数据、测试日志等证据整理至 `docs/progress/` 并在设计文档引用。
    - 与 README.md、AGENTS.md 核对，确保规范一致且说明 Mock→Real 开关位置。
  - Requirements: R3, R5（同步规范）
  - Acceptance:
    - Phase Gate 完成后提交包含 Phase Commit Block 的提交记录，所有文档与代码保持一致，审查时可追溯证据。
    - Notes: `docs/progress/markdown_media_sanitizer.md`、`docs/perf/markdown_sanitizer.md` 与 spec 同步。
