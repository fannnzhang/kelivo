# 📄 requirements.md — MarkdownMediaSanitizer 迁移到 Rust

## 1. Introduction（需求背景）

- **背景**: 当前应用中的 `MarkdownMediaSanitizer` 工具在处理内联 base64 图片和本地图片路径转换时，完全依赖 Dart 实现。这些操作涉及大量字符串处理、正则表达式匹配、base64 编解码以及文件 I/O，属于计算密集型任务。尤其在处理大尺寸图片或大量图片时，会占用主线程资源，影响 UI 流畅度。
- **目标**: 为了提升性能和响应速度，计划将 `MarkdownMediaSanitizer` 的核心逻辑从 Dart 迁移到 Rust。利用 Rust 的高性能特性，将密集计算任务转移到原生层执行，从而释放 Dart 线程，优化用户体验。

---

## 2. 需求描述（Requirements）

- **R1**: 将 `replaceInlineBase64Images` 函数的核心逻辑迁移到 Rust。
- **R2**: 将 `inlineLocalImagesToBase64` 函数的核心逻辑迁移到 Rust。
- **R3**: 确保迁移后的功能与原 Dart 实现完全兼容，输入输出行为保持一致。
- **R4**: 通过 `flutter_rust_bridge` (FRB) 实现 Dart 与 Rust 之间的通信。
- **R5**: 迁移后，Dart 层的 `MarkdownMediaSanitizer` 应作为调用 Rust 实现的接口。

---

## 3. 分阶段开发策略（Phased Development Strategy）

| Phase | 标题 | 简要说明 |
|-------|------|----------|
| Phase 1 | Rust 核心逻辑实现 | 在 Rust 端实现 `replace_inline_base64_images` 和 `inline_local_images_to_base64` 的核心功能，并编写单元测试。 |
| Phase 2 | FRB 集成与 Dart 调用 | 通过 FRB 创建 Dart 与 Rust 之间的桥接，并更新 Dart 端的 `MarkdownMediaSanitizer` 以调用 Rust 实现。 |
| Phase 3 | 性能验证与清理 | 对比迁移前后的性能，验证性能提升。清理遗留的 Dart 实现代码。 |

---

## 4. Requirements（详细需求）

### Phase 1: Rust 核心逻辑实现

#### Requirement 1: 实现 `replace_inline_base64_images`

User Story:
- 作为开发者，我希望在 Rust 中实现一个函数，该函数能够接收一个 Markdown 字符串，将其中的 base64 图片数据解码并保存为文件，然后将图片语法替换为文件路径，以便提升图片处理性能。

Acceptance Criteria:
- WHEN 输入一个包含 base64 图片的 Markdown 字符串 THEN Rust 函数 SHALL 正确解析出所有 base64 图片数据。
- WHEN 成功解码 base64 数据 THEN Rust 函数 SHALL 将解码后的二进制数据写入本地文件系统。
- WHEN 文件写入成功 THEN Rust 函数 SHALL 返回一个新的 Markdown 字符串，其中 base64 数据链接被替换为对应的文件路径。
- WHEN 输入的字符串不包含 base64 图片 THEN Rust 函数 SHALL 返回原始字符串。
- WHEN 处理过程中发生错误（如解码失败、文件写入失败） THEN Rust 函数 SHALL 能优雅地处理错误，并可将错误信息返回给调用方。

#### Requirement 2: 实现 `inline_local_images_to_base64`

User Story:
- 作为开发者，我希望在 Rust 中实现一个函数，该函数能够接收一个 Markdown 字符串，读取其中引用的本地图片文件，将其编码为 base64 数据，并替换原始文件路径，以便在需要时内联图片。

Acceptance Criteria:
- WHEN 输入一个包含本地图片路径的 Markdown 字符串 THEN Rust 函数 SHALL 正确解析出所有本地图片路径。
- WHEN 成功读取图片文件 THEN Rust 函数 SHALL 将文件内容编码为 base64 字符串。
- WHEN 编码成功 THEN Rust 函数 SHALL 返回一个新的 Markdown 字符串，其中文件路径被替换为对应的 base64 数据链接。
- WHEN 引用的文件不存在或读取失败 THEN Rust 函数 SHALL 保留原始的 Markdown 图片语法。
- WHEN 输入的字符串不包含本地图片路径 THEN Rust 函数 SHALL 返回原始字符串。

---

## 5. Non-functional & Cross-cutting（非功能与横切）

### 技术架构与代码规范
- **FRB 集成**: 遵循项目已有的 `flutter_rust_bridge` 配置和使用规范。
- **Rust 规范**: 遵循标准的 Rust 编码风格和最佳实践，确保代码的可读性和可维护性。
- **错误处理**: Rust 函数应返回 `Result` 类型，以便在 Dart 端能清晰地处理成功和失败的情况。

### 错误处理与用户体验
- **兼容性**: 迁移后的功能必须与现有行为完全一致，不能引入任何回归。
- **日志**: 在关键的迁移步骤（如文件 I/O、编解码）中添加适当的日志，便于调试。

### 性能与安全
- **性能基准**: 在迁移前后进行性能基准测试，量化性能提升。测试应覆盖不同大小和数量的图片场景。
- **安全性**: 确保文件操作的安全性，避免路径遍历等漏洞。处理文件路径时应进行充分的验证和清理。
