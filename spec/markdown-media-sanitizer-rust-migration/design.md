# 🧠 design.md — MarkdownMediaSanitizer 迁移 Rust 方案

## 0. 元信息（Meta）

- **Feature / Bug 名称**：MarkdownMediaSanitizer 核心逻辑迁移到 Rust
- **Spec 路径**：`spec/markdown-media-sanitizer-rust-migration/design.md`
- **版本 / 日期**：v1.0 · 2025-10-15
- **关联**：`requirements.md`、`tasks.md`
- **所有者 / 评审人**：Gemini / @kelivo
- **范围声明（Scope / Non-goals）**
  - **In Scope**：
    - 将 `MarkdownMediaSanitizer` 中的 `replaceInlineBase64Images` 和 `inlineLocalImagesToBase64` 函数的核心逻辑用 Rust 实现。
    - 通过 `flutter_rust_bridge` (FRB) 将 Rust 实现暴露给 Dart 端。
    - Dart 端的 `MarkdownMediaSanitizer` 更新为对 Rust 函数的调用封装。
  - **Out of Scope**：
    - 不改变 `MarkdownMediaSanitizer` 的现有接口和行为。
    - 不引入新的 Markdown 解析库，仅迁移现有逻辑。
    - 本次不涉及对 `ChatService` 中其他文件操作（如孤立文件清理）的迁移。

---

## 1. 项目基线与约束（Baseline & Constraints）

- **架构与平台**：应用为 Flutter App，已集成 `flutter_rust_bridge`，支持在 iOS, Android, macOS, Windows, Linux 平台调用 Rust 代码。
- **模块边界与现有约定**：`MarkdownMediaSanitizer` 是位于 `lib/utils/` 下的独立工具类，无复杂外部依赖。Rust 代码统一存放在 `rust/` 目录下。
- **外部依赖与环境**：Rust 端可依赖 `base64`, `regex`, `uuid` 等标准库或社区库来辅助实现。
- **关键约束**：
    - **性能**: 迁移后的实现必须比纯 Dart 版本有显著性能提升，尤其是在处理大文件和多文件场景。
    - **兼容性**: 必须保持与原功能 100% 的行为一致性。
- **假设 & 待确认**：假设现有的 FRB 设置能够顺利支持所需的数据类型（`String`, `Result<String, String>`)。

---

## 2. 目标与成功标准（Goals & Exit Criteria）

- **技术目标**：
  - 将计算密集型任务从 Dart 移至 Rust，降低 Dart Isolate 的负载。
  - 提升 Markdown 图片处理的执行效率。
- **退出标准**：
  - 所有在 `requirements.md` 中定义的验收标准均已满足。
  - 性能基准测试表明，新实现在目标场景下有至少 30% 的性能提升。
  - 代码通过所有 CI 检查，包括格式化、静态分析和单元测试。

---

## 3. 渐进式交付策略（Progressive Strategy）

| Phase | 目标 | 主要内容 | 依赖 | 演示与验收 | 回滚点 |
|------:|------|----------|------|------------|--------|
| 1 | Rust 核心逻辑实现 | 在 `rust/src/api/` 下创建新模块，实现 `replace_inline_base64_images` 和 `inline_local_images_to_base64` 的 Rust 版本。为这两个函数编写纯 Rust 的单元测试。 | `requirements.md` | 单元测试通过 | 移除 Rust 模块代码 |
| 2 | FRB 集成与 Dart 调用 | 在 FRB 的 `api.rs` 中暴露新的 Rust 函数。运行代码生成器。在 `lib/utils/markdown_media_sanitizer.dart` 中，修改原有函数实现，改为调用生成的 Rust 接口。 | Phase 1 | Dart 端的集成测试通过 | 恢复 `markdown_media_sanitizer.dart` 的原实现 |
| 3 | 性能验证与清理 | 编写基准测试，对比迁移前后在处理大 base64 字符串和多个本地图片时的性能差异。确认性能达标后，移除 Dart 中不再需要的旧逻辑代码。 | Phase 2 | 性能测试报告 | 若性能不达标，回滚到 Phase 2 的 Dart 实现 |

---

## 4. 方案概要（Solution Overview）

- **设计思路**：采用“桥接”模式，将 `MarkdownMediaSanitizer` 的 Dart 实现改造为调用层，核心处理逻辑下沉到 Rust。Dart 负责传递数据和接收结果，Rust 负责所有计算和文件操作。
- **影响面**：
  - `lib/utils/markdown_media_sanitizer.dart`: 实现将被替换。
  - `rust/src/api/`: 新增 Rust 模块。
  - `rust/src/frb_generated.rs`: 将被 FRB 工具更新。
- **兼容性/降级**：接口保持不变，对调用方透明。如果 Rust 实现失败，函数将返回错误，由 Dart 端处理，最差情况下可保留原始输入，确保应用不崩溃。
- **可观测性**：在 Rust 函数的关键路径（如文件读写失败）和 Dart 调用边界添加日志。

---

## 5. 模块与调用关系（Modules & Flows）

- **模块清单**
| 模块 | 职责 | 新增/修改/复用 |
|------|------|----------------|
| `rust/src/api/markdown_sanitizer.rs` | 实现 Markdown 图片处理的核心逻辑 | 新增 |
| `rust/src/api/mod.rs` | 暴露 `markdown_sanitizer` 模块 | 修改 |
| `lib/utils/markdown_media_sanitizer.dart` | 调用 Rust 函数，处理 Dart 与 Rust 的数据交互 | 修改 |
| `lib/config/feature_flags.dart` | Feature Flag 配置，控制 Mock/Real 切换 | 新增 |

- **核心调用链**：
  `Dart (UI/Service)` → `MarkdownMediaSanitizer.replace...()` → `RustLib.api.replace...()` → `Rust (markdown_sanitizer.rs)`

---

## 7. 合同与集成（Contracts & Integrations）

- **接口清单**
  | 名称 | 通信方式 | 请求 | 响应/负载 | 错误语义 |
  |------|----------|------|-----------|----------|
  | `replace_inline_base64_images` | FRB | `markdown: String` | `Result<String, String>` | `Err` 包含错误信息 |
  | `inline_local_images_to_base64` | FRB | `markdown: String` | `Result<String, String>` | `Err` 包含错误信息 |

---

## 9. 校验与验收（Verification & Acceptance）

- **测试层次**：
  - **单元测试**: 在 Rust 端对核心解析和文件操作逻辑进行测试。
  - **集成测试**: 在 Dart 端编写测试，确保 FRB 调用成功，且端到端行为符合预期。
  - **基准测试**: `test/benchmark/markdown_sanitizer_benchmark.dart` 评估 Mock 与 Real 性能，结果记录于 `docs/perf/markdown_sanitizer.md`。
- **关键用例表**
  | 编号 | 场景 | 步骤 | 期望 |
  |-----:|------|------|------|
  | 1 | 单个 Base64 图片 | 调用 `replaceInlineBase64Images` | 图片被存为文件，路径替换正确 |
  | 2 | 多个 Base64 图片 | 调用 `replaceInlineBase64Images` | 所有图片均被正确处理 |
  | 3 | 单个本地图片 | 调用 `inlineLocalImagesToBase64` | 图片被内联为 Base64 |
  | 4 | 混合内容 | 调用任一函数 | 仅目标图片被处理，其他内容不变 |
  | 5 | 错误处理 | 输入无效数据或模拟文件失败 | 函数返回 `Err`，Dart 端能捕获 |

---

## 14. 迁移与回滚（Migration & Rollback）

- **切换计划**：通过修改 `lib/utils/markdown_media_sanitizer.dart` 的实现来完成切换。无需复杂的版本控制或灰度。
- **回滚**：如果 Phase 2 或 3 出现问题，只需将 `lib/utils/markdown_media_sanitizer.dart` 的内容恢复到迁移前的版本即可完成回滚。

---

## 16. 风险与权衡（Risks & Trade-offs）

| 风险 | 影响 | 可能性 | 缓解 | 回滚触发 |
|------|------|--------|------|----------|
| FRB 集成问题 | 阻塞开发 | 低 | 遵循现有 FRB 实践，参考已有实现 | 无法在2天内解决 | 
| 性能不达标 | 未达到优化目标 | 中 | 在 Rust 端优化文件读写和正则性能 | 基准测试结果低于预期 |
| 平台兼容性问题 | 在特定平台（如 iOS 沙箱）出现文件路径问题 | 中 | 在 Rust 端和 Dart 端对路径进行严格测试和适配 | 特定平台测试失败 |

---

## 17. 代码组织与约定（Code Map & Conventions）

- **目录与命名**：
  - Rust 模块: `rust/src/api/markdown_sanitizer.rs`
  - Rust 函数: `replace_inline_base64_images`, `inline_local_images_to_base64`
- **注释与文档**：为公开的 Rust 函数添加文档注释。

---

## 18. 评审清单（Review Checklist）

- [x] 与 `requirements.md` 对齐
- [x] 每个 Phase 可运行/可测试/可回滚
- [x] 兼容性与风险明确
