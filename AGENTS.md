# Repository Guidelines

## Project Structure & Module Organization
Source lives in `lib/`, with `lib/main.dart` bootstrapping providers and feature folders (chat, settings, providers). Mirror that hierarchy in `test/` using `*_test.dart` files so coverage stays aligned with production code. UI assets and icons sit under `assets/`, while `lib/l10n/` stores ARB localization files generated into `lib/l10n/app_localizations.dart`. Platform-specific code for Android, iOS, macOS, Linux, Windows, and web is scoped to their respective top-level folders; touch these only when altering platform integrations or build settings.

## Build, Test, and Development Commands
Run `flutter pub get` after pulling to refresh dependencies. Use `flutter run -d <device-id>` for local development, and `flutter build apk` / `flutter build ios` for release artifacts. Keep the analyzer clean with `flutter analyze`, and let CI succeed locally by executing `flutter test` before pushing.

## Coding Style & Naming Conventions
Adopt the default Dart 2-space indentation and keep files in `lib/` and `test/` snake_cased (for example, `chat_history_view.dart`). Classes use PascalCase, members camelCase, and constants SCREAMING_SNAKE_CASE when top-level. Format every change with `dart format lib test` and rely on the `flutter_lints` ruleset defined in `analysis_options.yaml`; resolve warnings instead of suppressing them. Prefer `const` constructors where possible and scope helper widgets as private (`_WidgetName`) inside their defining files.

## Testing Guidelines
Add or update widget/unit tests in `test/`, grouping related cases with `group()` and naming the file after the feature (`chat_provider_test.dart`). Run `flutter test --coverage` when validating complex changes and keep new features covered by meaningful assertions, especially around provider logic and serialization. Skip network calls inside tests; stub async boundaries with fakes or test doubles.

## Commit & Pull Request Guidelines
Follow the conventional commit pattern seen in history (`feat:`, `fix:`, `chore:`, `refactor:`) with concise, imperative descriptions (`feat: add prompt variable picker`). Squash related work into logical commits and ensure each passes formatting, analyzer, and tests. Pull requests should describe the change, link the relevant issue or discussion, and include screenshots or screen recordings for UI updates. Mention any localization updates (ARB files or generated output) and flag build configuration changes so reviewers can test them quickly.

## Localization & Configuration Tips
Update `lib/l10n/app_en.arb` (and translations) when adding user-facing strings, then run `flutter gen-l10n` to regenerate bindings. Keep asset references in `pubspec.yaml` synchronized with files under `assets/`, and validate icon or splash changes via the existing `flutter_launcher_icons` and `flutter_native_splash` configurations before committing.

## Spec‑Kit Development Workflow
- Scope: Use for all non-trivial features/bugfixes to keep design, tasks, and code aligned.
- Read: `spec-kit.md` for end-to-end rules. Follow templates in `templates/requirements_template.md`, `templates/design_template.md`, `templates/tasks_template.md`.
- Branching:
  - `git checkout -b spec/<name>` for specs; follow up with `feature/<name>` for code.
- Required specs (per item):
  - `spec/<name>/requirements.md` — problem, constraints, acceptance.
  - `spec/<name>/design.md` — must include Phases and Phase Detail Blocks (goals/scope/deps & flags/implementation/acceptance & demo/testing/rollback/risks/impacts/exit checklist).
  - `spec/<name>/tasks.md` — executable tasks mapped to phases; status markers `[ ]` not started, `[-]` in progress, `[x]` done; each task includes Req/Design refs, file paths, acceptance, commands, rollback.
- Acceptance gates before merge:
  - Exit checklist in each Phase is satisfied; specs kept in sync with code changes.
  - Repo checks pass: `flutter analyze`, `flutter test`, `dart format lib test`.
- PR expectations:
  - Link the spec directory, summarize phases and current status, attach screenshots for UI changes.
