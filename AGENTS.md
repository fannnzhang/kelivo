# AGENTS.md — Spec-Driven Mobile Development Guidelines

Scope: Entire repository. Applies to generating and maintaining docs under `spec/` and templates under `templates/`.

Principles
- Templates must be pure skeletons. Do not embed usage guidance, examples, or process constraints inside files in `templates/`. Place all rules, conventions, and constraints here.
- Doc-driven, Mock-first, Progressive delivery. Implement phased, verifiable increments that move from local mock to real integration with clear switches.
- Avoid over-engineering. Design to current needs and project constraints.

Spec Workflow
1) Create/Update `spec/<feature-or-bug>/requirements.md`, `design.md`, `tasks.md` by instantiating the templates in `templates/`.
2) If a spec exists, perform incremental updates without breaking section order or numbering.
3) Always keep content aligned with repository `README.md`, any `constitution.md`, and existing code.

Templates Usage Rules
1) requirements.md
- Write requirements using EARS syntax:
  WHEN <trigger> THEN <system> SHALL <response> [SO THAT <rationale>]
- Split work into Phases in the “Phased Development Strategy” table. Each Phase is independently runnable, testable, and revertible.
- For each Phase, author detailed Requirements with:
  - User Story (role, action, purpose)
  - Acceptance Criteria (EARS lines)
- Capture Non-functional & Cross-cutting requirements that apply across phases (architecture constraints, error handling/UX, performance/security). Keep content concrete and testable.

2) design.md
- Baseline from the current repo. Do not assume architecture; infer from code/docs. If unknowns exist, mark as “To Confirm” with an evidence path and deadline.
- Provide a progressive strategy (Phase table) that maps to requirements Phases and supports Mock → Real with an explicit switch (config/DI/build flag/env).
- Describe solution minimally on top of the baseline: what’s new/changed/reused; compatibility; observability.
- Document modules and call flows, data models and DTO↔Domain mapping, contracts/integrations, UI/interaction, verification & acceptance, performance, security & privacy, observability & ops, impact, migration & rollback, release, risks & trade-offs, code map & conventions, review checklist.

3) tasks.md
- Purpose: definitive, executable implementation plan (final authority for work). Keep design prose out.
- Structure: nested, numbered checklist only (no tables)
  - Phases Overview (bulleted list of Phase titles and brief notes)
  - Phase sections (`## Phase N: <Title>`), each with numbered tasks:
    - Task format:
      - `- [ ] N. <Task group>`
      - Sub-bullets: Summary, Files, Changes, Requirements, Acceptance
      - Optional subtasks: `N.1`, `N.2`, … with the same sub-bullets
  - Cross-phase Tasks section for global items spanning all phases
- Guidance: each task must include exact file paths and concrete changes; acceptance describes how to verify completion.

Execution Policy (applies when filling tasks.md)
- Mock-first, then Real: deliver an end-to-end flow with local mock or stub before real integration; keep a clear switch/flag.
- Execution order inside a user story: tests-first (if present) → models → services → endpoints/UI → integration → validation/logging.
- Parallelism: tasks marked [P] can run in parallel; user stories can run in parallel after Foundational Phase completes.

Branching & Commits
- Create a working branch per spec item: `git checkout -b spec/<feature-or-bug-name>`
- Phase Gate (mandatory): After completing Implementation for a Phase:
  - Update `tasks.md` statuses and the Progress Log
  - Attach acceptance evidence (screenshots, recordings, logs, API receipts)
  - Verify build/compile passes
    - Project default: `fvm dart format --set-exit-if-changed . && fvm dart analyze`
    - If enabled, run minimal build: `fvm flutter build apk --debug` (or iOS equivalent)
  - Create a dedicated commit
    - Examples:
      - `feat(<scope>): complete Phase <n> – <title> [Mock|Real]`
      - `fix(<scope>): stabilize Phase <n> – <title>`
    - Reference related Spec/Issue where applicable (e.g., `refs: SPEC-123`)

## MCP Usage (context7 MCP)
- All external tool access and context retrieval must use MCP (Model Context Protocol).
- Required provider: `context7`. Configure and connect to the context7 MCP server for development and CI.
- Prefer MCP tools over direct network calls. In specs/tasks, name the MCP tool/capability being used.
- If MCP is unavailable, mark as “To Confirm” with fallback and deadline, proceed Mock‑first, and avoid hardcoding tokens.
- Secrets/endpoints must come from environment variables (e.g., `MCP_CONTEXT7_URL`, `MCP_CONTEXT7_TOKEN`) and must not be committed.
    
# Repository Guidelines

## Project Structure & Module Organization
Source lives in `lib/`, with `lib/main.dart` bootstrapping providers and feature folders (chat, settings, providers). Mirror that hierarchy in `test/` using `*_test.dart` files so coverage stays aligned with production code. UI assets and icons sit under `assets/`, while `lib/l10n/` stores ARB localization files generated into `lib/l10n/app_localizations.dart`. Platform-specific code for Android, iOS, macOS, Linux, Windows, and web is scoped to their respective top-level folders; touch these only when altering platform integrations or build settings.

## Build, Test, and Development Commands
Run `fvm flutter pub get` after pulling to refresh dependencies. Use `fvm flutter run -d <device-id>` for local development, and `fvm flutter build apk` / `fvm flutter build ios` for release artifacts. Keep the analyzer clean with `fvm dart analyze`, and let CI succeed locally by executing `fvm flutter test` before pushing.

## Coding Style & Naming Conventions
Adopt the default Dart 2-space indentation and keep files in `lib/` and `test/` snake_cased (for example, `chat_history_view.dart`). Classes use PascalCase, members camelCase, and constants SCREAMING_SNAKE_CASE when top-level. Format every change with `fvm dart format lib test` and rely on the `flutter_lints` ruleset defined in `analysis_options.yaml`; resolve warnings instead of suppressing them. Prefer `const` constructors where possible and scope helper widgets as private (`_WidgetName`) inside their defining files.

## Testing Guidelines
Add or update widget/unit tests in `test/`, grouping related cases with `group()` and naming the file after the feature (`chat_provider_test.dart`). Run `fvm flutter test --coverage` when validating complex changes and keep new features covered by meaningful assertions, especially around provider logic and serialization. Skip network calls inside tests; stub async boundaries with fakes or test doubles.

## Commit & Pull Request Guidelines
Follow the conventional commit pattern seen in history (`feat:`, `fix:`, `chore:`, `refactor:`) with concise, imperative descriptions (`feat: add prompt variable picker`). Squash related work into logical commits and ensure each passes formatting, analyzer, and tests. Pull requests should describe the change, link the relevant issue or discussion, and include screenshots or screen recordings for UI updates. Mention any localization updates (ARB files or generated output) and flag build configuration changes so reviewers can test them quickly.

## Localization & Configuration Tips
Update `lib/l10n/app_en.arb` (and translations) when adding user-facing strings, then run `fvm flutter gen-l10n` to regenerate bindings. Keep asset references in `pubspec.yaml` synchronized with files under `assets/`, and validate icon or splash changes via the existing `flutter_launcher_icons` and `flutter_native_splash` configurations before committing.

## Spec‑Kit Development Workflow
- Scope: Use for all non-trivial features/bugfixes to keep design, tasks, and code aligned.
- Follow templates in `templates/requirements_template.md`, `templates/design_template.md`, `templates/tasks_template.md`.
- Branching:
  - `git checkout -b spec/<name>` for specs; follow up with `feature/<name>` for code.
- Required specs (per item):
  - `spec/<name>/requirements.md` — problem, constraints, acceptance.
  - `spec/<name>/design.md` — must include Phases and Phase Detail Blocks (goals/scope/deps & flags/implementation/acceptance & demo/testing/rollback/risks/impacts/exit checklist).
  - `spec/<name>/tasks.md` — executable tasks mapped to phases; status markers `[ ]` not started, `[-]` in progress, `[x]` done; each task includes Req/Design refs, file paths, acceptance, commands, rollback.
- Acceptance gates before merge:
  - Exit checklist in each Phase is satisfied; specs kept in sync with code changes.
  - Repo checks pass: `fvm dart analyze`, `fvm flutter test`, `fvm dart format lib test`.
- PR expectations:
  - Link the spec directory, summarize phases and current status, attach screenshots for UI changes.
