# Spec Workflow(强制执行 非常重要)

本规范定义了新功能或 Bug 修复从需求到任务落地的完整流程。**每一个 spec 必须对应一个独立的 Git 分支**：`spec/<spec-name>`
该分支承载本次改动的规范产出与实现步骤，所有改动均围绕此分支进行。

---

## 🧭 总则

- **一 Spec 一分支**：始终使用 `spec/<spec-name>` 命名，独立于主分支。
- **三文件完整性**：`requirements.md`、`design.md`、`tasks.md` 必须齐全。
- **模板驱动**：所有 spec 文档必须从 `specify/templates/` 模板实例化。
- **结构不可破坏**：已有章节和编号必须保持稳定。
- **全局一致性**：与 `README.md`、`constitution.md` 和代码保持一致。

## 📁 1. 创建 / 更新 Spec 文档

为每个新功能或修复在 `specs/<feature-or-bug>/` 下创建以下文件：

- `requirements.md`：描述需求与约束条件
- `design.md`：整体设计方案和决策依据
- `tasks.md`：任务分解、阶段性检查点和交付定义（DoD）

创建内容应基于模板：
```
specify/templates/requirements.md
specify/templates/design.md
specify/templates/tasks.md
```
> 📌 每个 spec 的落地分为以上阶段，所有阶段均需输出对应文档 如果对应 spec 已存在，需进行**增量更新**，不得打乱章节结构或编号。

---

## 🔁 2. 增量演进

- 更新时保持各文件章节顺序和结构稳定，不得随意删除或重排。
- 内容变更必须与代码实现及其他文档保持一致性：
    - `README.md`
    - `constitution.md`
    - 现有代码逻辑

---

## ✅ 3. 分支与提交规范

- 所有 spec 都必须独立在 `spec/<spec-name>` 分支中完成。
- 分支内包含三份文档：`requirements.md`、`design.md`、`tasks.md`。
- 所有提交必须直接更新这些文件，不得绕过规范产物。

---

1) Create/Update `specs/<feature-or-bug>/requirements.md`, `design.md`, `tasks.md` by instantiating the templates in `specify/templates/`.
2) If a spec exists, perform incremental updates without breaking section order or numbering.
3) Always keep content aligned with repository `README.md`, any `constitution.md`, and existing code.

## Templates Usage Rules

### requirements.md
- Write requirements using EARS syntax:
  WHEN <trigger> THEN <system> SHALL <response> [SO THAT <rationale>]
- Split work into Phases in the “Phased Development Strategy” table. Each Phase is independently runnable, testable, and revertible.
- For each Phase, author detailed Requirements with:
    - User Story (role, action, purpose)
    - Acceptance Criteria (EARS lines)
- Capture Non-functional & Cross-cutting requirements that apply across phases (architecture constraints, error handling/UX, performance/security). Keep content concrete and testable.

### design.md
- Baseline from the current repo. Do not assume architecture; infer from code/docs. If unknowns exist, mark as “To Confirm” with an evidence path and deadline.
- Provide a progressive strategy (Phase table) that maps to requirements Phases and supports Mock → Real with an explicit switch (config/DI/build flag/env).
- Describe solution minimally on top of the baseline: what’s new/changed/reused; compatibility; observability.
- Document modules and call flows, data models and DTO↔Domain mapping, contracts/integrations, UI/interaction, verification & acceptance, performance, security & privacy, observability & ops, impact, migration & rollback, release, risks & trade-offs, code map & conventions, review checklist.

### tasks.md
- Purpose: definitive, executable implementation plan (final authority for work). Keep design prose out.
- Structure: nested, numbered checklist only (no tables)
    - Phases Overview (bulleted list of Phase titles and brief notes)
    - Phase sections (`## Phase N: <Title>`), each with numbered tasks:
        - Task format:
            - `- [ ] N. <Task group>`
            - task status: []default [-] doing [x] done
            - Sub-bullets: Summary, Files, Changes, Requirements, Acceptance
            - Optional subtasks: `N.1`, `N.2`, … with the same sub-bullets
    - Cross-phase Tasks section for global items spanning all phases
- Guidance: each task must include exact file paths and concrete changes; acceptance describes how to verify completion.

### Execution Policy (applies when filling tasks.md)
- Mock-first, then Real: deliver an end-to-end flow with local mock or stub before real integration; keep a clear switch/flag.
- Execution order inside a user story: tests-first (if present) → models → services → endpoints/UI → integration → validation/logging.
- Parallelism: tasks marked [P] can run in parallel; user stories can run in parallel after Foundational Phase completes.