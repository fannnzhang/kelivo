# 🧠 design.md — 统一文本提取服务方案设计

## 0. 元信息（Meta）

- **Feature 名称**: `feat(rust): unified-document-text-extractor`
- **Spec 路径**: `spec/document-extractor-rust-migration/design.md`
- **版本 / 日期**: v1.0 · 2025-10-15
- **关联**: `requirements.md`
- **范围声明**
  - **In Scope**: 重构 `DocumentTextExtractor`，将其所有文件解析逻辑（PDF, DOCX, Plain Text）迁移到 Rust。
  - **Out of Scope**: 支持加密的文档、图片 OCR、除 `extract` 方法之外的任何其他功能。

---

## 4. 方案概要（Solution Overview）

- **设计思路**: 将 `DocumentTextExtractor` 的角色从“执行者”转变为“调度者”。Dart 层仅负责根据 MIME 类型判断文件格式，然后将文件路径传递给 Rust 核心库。所有繁重的解压缩、解析和文本提取工作都在 Rust 中完成，为所有平台提供统一、高效的实现。
- **影响面**: 
  - **修改**: `lib/core/services/chat/document_text_extractor.dart`
  - **新增**: `rust/src/api/document_parser.rs`
  - **移除**: Dart 依赖 `read_pdf_text`, `archive`, `xml`。

---

## 5. 模块与调用关系（Modules & Flows）

- **模块清单**
  | 模块 | 职责 | 新增/修改/复用 |
  |---|---|---|
  | `document_text_extractor.dart` | 根据 MIME 类型调度到对应的 Rust 函数 | 修改 |
  | `rust/src/api/document_parser.rs` | 实现 PDF, DOCX, Plain Text 的文本提取 | 新增 |

- **核心调用链**: 
  `Dart:extract(path, mime)` → `(switch on mime)` → `Rust:extract_text_from_pdf(path)` OR `Rust:extract_text_from_docx(path)` OR `Rust:read_text_fallback(path)` → `Return String`

---

## 7. 合同与集成（Contracts & Integrations）

- **接口/事件清单 (FRB)**
  | 名称 | 请求 (参数) | 响应/负载 | 错误语义 |
  |---|---|---|---|
  | `extract_text_from_pdf` | `path: String` | `anyhow::Result<String>` | 返回包含解析错误的 `Err` |
  | `extract_text_from_docx` | `path: String` | `anyhow::Result<String>` | 返回包含解压或 XML 解析错误的 `Err` |
  | `read_text_fallback` | `path: String` | `anyhow::Result<String>` | 返回包含文件读取或解码错误的 `Err` |

- **接口示例（Rust 侧）**
  ```rust
  // In rust/src/api/document_parser.rs

  pub fn extract_text_from_pdf(path: String) -> anyhow::Result<String> {
      // Implemented using `pdf_extract` crate
  }

  pub fn extract_text_from_docx(path: String) -> anyhow::Result<String> {
      // Implemented using `zip` and `quick-xml` crates
  }

  pub fn read_text_fallback(path: String) -> anyhow::Result<String> {
      // Implemented using `std::fs::read_to_string`
  }
  ```

---

## 9. 校验与验收（Verification & Acceptance）

- **测试层次**: 单元测试 (Rust), 集成测试 (Flutter)
- **关键用例表**
  | 编号 | 场景 | 步骤 | 期望 | 验收方式 |
  |---:|---|---|---|---|
  | 1.1 | Rust PDF 解析 | 调用 `extract_text_from_pdf` 并传入一个有效的 PDF 文件路径 | 返回提取的文本 | `cargo test` |
  | 1.2 | Rust DOCX 解析 | 调用 `extract_text_from_docx` 并传入一个有效的 DOCX 文件路径 | 返回提取的文本 | `cargo test` |
  | 2.1 | Flutter 集成测试 | 调用 `DocumentTextExtractor.extract` 并传入 PDF 文件 | 成功返回文本内容 | `flutter test` |
  | 2.2 | Flutter 集成测试 | 调用 `DocumentTextExtractor.extract` 并传入 DOCX 文件 | 成功返回文本内容 | `flutter test` |
  | 2.3 | Flutter 集成测试 | 调用 `DocumentTextExtractor.extract` 并传入 TXT 文件 | 成功返回文本内容 | `flutter test` |

---

## 11. 安全与隐私（Security & Privacy）

- **文件访问**: Rust 函数接收文件路径进行操作。Dart 端在调用前应确保路径的合法性，避免将敏感的系统文件路径传递给解析器。
