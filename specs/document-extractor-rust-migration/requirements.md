# 📄 requirements.md — 统一文本提取服务迁移至 Rust

## 1. Introduction（需求背景）

- **当前实现**: `DocumentTextExtractor` 使用混合策略提取文本：对 PDF 文件，它依赖一个仅支持 iOS/Android 的 Flutter 插件 (`read_pdf_text`)；对 DOCX 文件，它使用纯 Dart 的 `archive` 和 `xml` 包进行解析。这种实现导致了平台支持不完整、依赖分散和潜在的性能瓶颈。
- **迁移目标**: 将所有核心文本提取逻辑（包括 PDF, DOCX, 和纯文本）统一迁移到 Rust 中实现。目标是创建一个单一、高性能、完全跨平台的文本提取服务，并简化 Dart 端的代码和依赖。

---

## 2. 需求描述（Requirements）

- **R1**: 在 Rust 中提供三个独立的函数，分别用于从 PDF、DOCX 和通用文本文档中提取文本内容。
- **R2**: Dart 端的 `DocumentTextExtractor.extract` 方法必须被重构为一个纯粹的调度器，根据文件 MIME 类型调用对应的 Rust 函数。
- **R3**: 移除对 Flutter 插件 `read_pdf_text` 的依赖。
- **R4**: 移除对 Dart 包 `archive` 和 `xml` 的依赖（假设它们未在项目其他地方使用）。
- **R5**: 新的实现必须能在所有 Flutter 支持的平台上（Mobile, Desktop, Web）提供一致的文本提取功能。

---

## 4. Requirements（详细需求）

### Phase 1: 统一实现

#### Requirement 1.1: 实现 Rust 文本提取函数

User Story:
- 作为一名开发者，我希望在 Rust 中实现一套完整的文件文本提取函数，以便为所有平台提供统一、高效的后端服务。

Acceptance Criteria:
- [ ] Rust 库需提供 `extract_text_from_pdf` 函数，使用 `pdf-extract` crate 实现。
- [ ] Rust 库需提供 `extract_text_from_docx` 函数，使用 `zip` 和 `quick-xml` crate 实现。
- [ ] Rust 库需提供 `read_text_fallback` 函数，用于读取纯文本文件。
- [ ] 所有函数都必须通过 `flutter_rust_bridge` 暴露给 Dart，并使用 `anyhow::Result` 进行错误处理。

#### Requirement 1.2: 重构 Dart 调度器并移除依赖

User Story:
- 作为一名开发者，我希望简化 Dart 端的 `DocumentTextExtractor`，使其只负责根据文件类型调用相应的 Rust 函数，并移除所有旧的、平台相关的依赖。

Acceptance Criteria:
- [ ] `DocumentTextExtractor.extract` 方法内部逻辑被一个 `switch` 或 `if/else if` 结构取代，该结构根据 `mime` 参数调用对应的 FRB 函数。
- [ ] `pubspec.yaml` 中不再包含 `read_pdf_text`, `archive`, `xml` 的依赖项。
- [ ] 整个文本提取功能在所有目标平台上都能正常工作，并通过集成测试的验证。

---

## 5. Non-functional & Cross-cutting（非功能与横切）

- **性能与安全**: Rust 实现应显著快于原有的 Dart DOCX 解析，并与原 PDF 插件性能相当或更好。所有外部文件路径在传递给 Rust 前都应经过校验。
- **错误处理**: Rust 函数返回的 `Err` 必须包含足够的信息，以便 Dart 端进行调试和向用户反馈（例如，“文件损坏”或“格式不支持”）。
