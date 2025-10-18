# 🛠 tasks.md — 统一文本提取服务任务清单

## 分阶段开发策略（Phases Overview）

- **Phase 1: Rust 实现与单元测试** (在 Rust 端独立完成所有文件格式的解析与测试)
- **Phase 2: Dart 集成与依赖移除** (重构 Dart 代码，移除旧的插件和包)
- **Phase 3: 清理与收尾** (格式化、审查、最终确认)

---

## Phase 1: Rust 实现与单元测试

- [x] **1. 环境搭建与模块定义**
  - **Summary**: 创建 Rust 模块并添加 `pdf-extract`, `zip`, `quick-xml` 等依赖。
  - **Files**: `rust/Cargo.toml`, `rust/src/api/mod.rs`, `rust/src/api/document_parser.rs`
  - **Acceptance**: 项目可以成功编译。

- [x] **2. 实现 PDF 提取**
  - **Summary**: 实现 `extract_text_from_pdf` 函数并编写单元测试。
  - **Files**: `rust/src/api/document_parser.rs`
  - **Acceptance**: `cargo test` 通过 PDF 相关的测试用例。

- [x] **3. 实现 DOCX 提取**
  - **Summary**: 实现 `extract_text_from_docx` 函数并编写单元测试。
  - **Files**: `rust/src/api/document_parser.rs`
  - **Acceptance**: `cargo test` 通过 DOCX 相关的测试用例。

- [x] **4. 实现纯文本回退**
  - **Summary**: 实现 `read_text_fallback` 函数并编写单元测试。
  - **Files**: `rust/src/api/document_parser.rs`
  - **Acceptance**: `cargo test` 通过纯文本读取相关的测试用例。

---

## Phase 2: Dart 集成与依赖移除

- [x] **1. 生成 Bridge 代码并重构 Dart**
  - **Summary**: 运行 FRB 代码生成器，并重构 `DocumentTextExtractor` 为调度器。
  - **Files**: `lib/core/services/chat/document_text_extractor.dart`
  - **Changes**:
    - 运行 `flutter_rust_bridge_codegen`。
    - `extract` 方法内部逻辑替换为根据 `mime` 调用不同 Rust 函数的 `switch` 结构。
  - **Acceptance**: Dart 代码编译通过，新逻辑替换旧逻辑。

- [x] **2. 移除旧依赖**
  - **Summary**: 从 `pubspec.yaml` 中移除不再需要的插件和包。
  - **Files**: `pubspec.yaml`
  - **Changes**:
    - 运行 `dart pub remove read_pdf_text`。
    - 运行 `dart pub remove archive`。
    - 运行 `dart pub remove xml`。
  - **Acceptance**: 依赖被成功移除，项目仍可正常编译运行。

- [x] **3. 运行 Flutter 集成测试**
  - **Summary**: 验证重构后的端到端流程。
  - **Files**: `test/...`
  - **Acceptance**: 覆盖 PDF, DOCX, TXT 的集成测试全部通过。

---

## Phase 3: 清理与收尾

- [x] **1. 代码格式化与审查**
  - **Summary**: 确保所有新代码符合项目规范。
  - **Files**: 所有修改过的文件。
  - **Acceptance**: `dart format`, `cargo fmt`, `flutter analyze`, `cargo clippy` 均无问题。

- [ ] **2. 最终评审**
  - **Summary**: 对所有修改进行最终的代码评审。
  - **Acceptance**: PR 被合并。
