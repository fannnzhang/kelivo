# AGENTS.md — Spec-Driven Mobile Development Guidelines

## Spec Driven workflow(非常重要 强制执行)
Scope: Entire repository. Applies to generating and maintaining docs under `specs/` and templates under `specify/templates/`.

1) Create/Update `specs/<feature-or-bug>/requirements.md`, `design.md`, `tasks.md` by instantiating the templates in `specify/templates/`.
2) If a spec exists, perform incremental updates without breaking section order or numbering.
3) Always keep content aligned with repository `README.md`, any `constitution.md`, and existing code.
4) Always checkout (-b) the `specs/<feature-or-bug>` branch and execute the `tasks.md`

Principles
- Templates must be pure skeletons. Do not embed usage guidance, examples, or process constraints inside files in `specify/templates/`. Place all rules, conventions, and constraints here.
- Doc-driven, Mock-first, Progressive delivery. Implement phased, verifiable increments that move from local mock to real integration with clear switches.
- Avoid over-engineering. Design to current needs and project constraints.

---

## MCP Usage (context7 MCP)
- All external tool access and context retrieval must use MCP (Model Context Protocol).
- Required provider: `context7`. Configure and connect to the context7 MCP server for development and CI.
- Prefer MCP tools over direct network calls. In specs/tasks, name the MCP tool/capability being used.
- If MCP is unavailable, mark as “To Confirm” with fallback and deadline, proceed Mock‑first, and avoid hardcoding tokens.
- Secrets/endpoints must come from environment variables (e.g., `MCP_CONTEXT7_URL`, `MCP_CONTEXT7_TOKEN`) and must not be committed.

---

## Alerter Local Notification(非常重要)
在任务执行各阶段，**强制必须**使用 `alerter` 发送 macOS 通知提醒用户进度、完成状态或请求输入。  
通知类型参见`Alerter-Usage.md`，包括：

- 普通通知（任务开始/完成）发送一次即可 **无需关注结果**
- 确认型通知（actions）发送一次即可 **强制等待结果**
- 回复型通知（reply）发送一次即可 **强制等待结果**

---

# 📁 Project Structure & Module Organization

When generating **code, designs, or technical solutions**, strictly follow the existing architecture, module layout, and tech stack defined in `README.md` and `constitution.md`.

---

## 📌 Core Rules

- All decisions on structure, naming, dependencies, and interfaces **must align** with current project conventions.
- Do **not** introduce new patterns, layers, or technologies not present in the existing stack.
- All new modules, files, and directories must follow the current hierarchy and naming style.

---


## 🧠 Model Instructions

Before generating output for any of these tasks:

- ✏️ **Design** — base architecture and component design on the patterns in `README.md` and `constitution.md`.
- 🛠 **Code changes** — follow current module boundaries, file naming, and dependency usage.
- 🧩 **New features** — extend within the existing structure rather than creating new parallel systems.
- 📐 **Spec writing** — ensure references in `requirements.md`, `design.md`, and `tasks.md` map to actual modules and paths.
