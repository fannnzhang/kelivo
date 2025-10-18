# Branching & Commits(重要且强制)

- Language: Use English for commit messages (title and body).
- 标题：`type(scope): summary`（简洁明确）
- 分隔：标题与正文之间空一行
- Body: use real newlines; NEVER embed literal `\n` to represent line breaks
- Structure: optional section headings followed by bullets, e.g., `Templates:`, `Docs:`, `Spec:`
- Bullets: one item per line starting with `- `
- Wrapping: aim for ~72 chars per line for readability

Phase Commit Block (must be included in commit body)
- Phase: `<n> – <title> [Mock|Real]`
- Spec: `spec/<feature-or-bug>/tasks.md`
- Tasks: `<IDs completed>`


Example
```
feat(templates+spec): introduce clean templates and checklist structure; add AGENTS.md; align visit-review-rejection-flow spec

Templates:
- Merge Speckit-style checklist into the tasks template
- All templates now contain only structural skeletons (guidance moved to AGENTS.md)
- Simplify Definition of Done (DoD) in tasks.md

Docs:
- Add AGENTS.md: includes template usage guide, Mock → Real transition guide, and Phase Gate commit rules
- Add commit message convention: enforce real newlines (never literal "\n"), use title + list structure

Spec:
- Align visit-review-rejection-flow requirements/design/tasks with the new template structure
- Preserve existing content, remove template-style guidance, and add phase-level checklist in tasks.md
```