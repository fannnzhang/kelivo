# 📄 requirements.md — rust_lib_Kelivo Loader 修复

## 1. Introduction（需求背景）

- Flutter App 在启用真实 Rust 能力 (`USE_RUST=true`) 时调用 `RustLib.init()` 立即崩溃，日志提示 `Invalid argument(s): Failed to load dynamic library 'rust_lib_kelivo.framework/rust_lib_kelivo'`。
- Rust FFI crate 名称及各平台打包脚手架统一为 `rust_lib_Kelivo`（参见 `rust/api/Cargo.toml`、`rust_builder/*`），但现有 FRB 生成代码仍尝试加载小写 `rust_lib_kelivo`，导致 iOS/macOS 运行期无法定位动态库。
- 需求目标：恢复所有目标平台（iOS、macOS、Android、桌面）在真实模式下成功加载 Rust FFI，并保持 mock 切换能力与现有 README / docs 约定一致。

---

## 2. 需求描述（Requirements）

- R1 运行期加载：`RustLib.init()` 在真实模式下 SHALL 成功加载 `rust_lib_Kelivo` 对应动态/静态库，无 `dlopen` 或链接失败。
- R2 生成同步：Flutter 侧 FRB 绑定 SHALL 与当前 Rust crate 命名保持一致，可通过 `flutter_rust_bridge_codegen generate` 自动生成。
- R3 验证路径：`fvm flutter test`、关键集成测试及手动运行 SHALL 在真实模式通过，README 与 `docs/rust_frb_build.md` 的流程不被破坏。
- R4 可回退：保留 `USE_RUST=false` mock 流程，初始化逻辑 SHALL 继续支持 `RustLib.initMock(api: ...)`。

---

## 3. 分阶段开发策略（Phased Development Strategy）

| Phase | 标题 | 简要说明 |
|-------|------|----------|
| Phase 1 | Loader 修正 | 统一 FRB 动态库 `stem` 与 crate 名称，确保默认加载路径正确。 |
| Phase 2 | 生成与验证 | 重新生成 FRB 绑定，跑通静态检查与集成测试，验证 iOS/macOS 运行。 |
| Phase 3 | 文档与交付 | 更新相关文档、任务追踪与回归验证记录，确保团队协作一致性。 |

---

## 4. Requirements（详细需求）

### Phase 1: Loader 修正

#### Requirement R1: Align FRB loader stem

User Story:
- 作为 iOS/macOS 开发者，我希望 `RustLib.init()` 自动指向 `rust_lib_Kelivo`，以便应用在真实模式下立即启动。

Acceptance Criteria:
- `lib/src/rust/frb_generated.dart` 中 `ExternalLibraryLoaderConfig.stem` 更新为 `rust_lib_Kelivo`，与 Rust crate 及脚手架一致。
- 再次运行 `USE_RUST=true` 构建时不再出现 `rust_lib_kelivo` 相关 `dlopen` 错误。

---

### Phase 2: 生成与验证

#### Requirement R2: Regenerate FRB bindings

User Story:
- 作为项目维护者，我需要通过官方流程 (`flutter_rust_bridge_codegen generate`) 重建绑定，以便后续更新保持自动化。

Acceptance Criteria:
- `flutter_rust_bridge_codegen generate` 成功执行（可在 MCP context7 工具链记录执行时间）。
- 生成文件（`lib/src/rust/*`）与新的 loader 设置一致，无遗留小写命名。

#### Requirement R3: Validate real-mode boot

User Story:
- 作为 QA，我希望在真实模式下运行集成测试和关键手测，确认应用稳定。

Acceptance Criteria:
- `fvm dart analyze`、`fvm flutter test` 全部通过。
- 至少一次 `fvm flutter run --dart-define=USE_RUST=true`（或通过 context7 远程流水线模拟）启动成功并记录。
- 如需跳过实际设备验证，标记 “To Confirm” 并给出截止时间与负责人。

---

### Phase 3: 文档与交付

#### Requirement R4: Update docs & guidance

User Story:
- 作为协作成员，我需要了解修复后的加载方式与验证记录，以便后续开发遵循统一流程。

Acceptance Criteria:
- `docs/rust_frb_build.md` 或相关 README 核对后确认无需更新；若需要，完成相应增补。
- 在 `specs/rust-lib-loader-fix/tasks.md` 中标记完成情况，并提及 MCP/context7 使用记录或待确认事项。
- 所有更改遵循现有命名与目录规范，无新技术栈引入。

---

## 5. Non-functional & Cross-cutting（非功能与横切）

### 技术架构与代码规范
- 遵循当前 Flutter + Provider + FRB 架构，保持 `lib/main.dart` 初始化流程与 `docs/rust_frb_build.md` 描述一致。
- 生成文件的修改通过官方 codegen 流程完成，避免手动偏差。

### 错误处理与用户体验
- 启动期若仍失败，应提供清晰日志指向新的库名，并在任务文档中记录排查步骤。
- Mock 模式 (`USE_RUST=false`) 行为保持不变，避免影响无 Rust 依赖场景。

### 性能与安全
- 无新增性能或安全要求；确保未引入硬编码路径或敏感信息。
- 继续要求从环境变量（如 `MCP_CONTEXT7_URL`、`MCP_CONTEXT7_TOKEN`）获取外部服务配置。
