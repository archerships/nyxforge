# NyxForge Token Economy Checklist

Purpose: reduce Codex/AI token usage while preserving architectural quality,
review quality, and continuity between sessions.

This checklist is operational guidance. It does not override
`doc/00_AGENTS.md`, `doc/00_MVP.md`, or direct user instructions.

## 1. Checklist for the user

Use this when starting or steering an AI session.

### 1.1 Start with the narrowest useful task

- [ ] Say whether the task is review-only, edit/apply, regenerate, test, or explain.
- [ ] Name the target surface: spec, CLI, UI mockups, data format, Rust crate, Flutter UI, tests, or logs.
- [ ] Name the files when known.
- [ ] Say whether generated outputs should be changed or only source generators.

Good examples:

- "Review `doc/05_TECH/file-format-spec.md` against `doc/00_MVP.md`; no edits."
- "Update `bin/bounty-market` and regenerate only its active mockup."
- "Search active mockup generators for stale oracle/bounty terminology; report findings first."

Avoid broad prompts unless a phase-gate audit is intended:

- "Review the whole app."
- "Read everything and tell me what is wrong."
- "Make the docs consistent."

### 1.2 Provide current context, not old context

- [ ] Point the agent to `doc/01_STATE.md` first.
- [ ] Mention the current decision or conflict being resolved.
- [ ] Tell the agent which previous response ID matters, if any.
- [ ] Avoid asking the agent to reread long historical logs unless the history itself is the task.

### 1.3 Keep source-of-truth boundaries clear

- [ ] For product behavior, use `doc/00_MVP.md`.
- [ ] For plugin behavior, use `doc/05_TECH/plugin-architecture.md`.
- [ ] For file schemas, use `doc/05_TECH/file-format-spec.md`.
- [ ] For UI mockups, prefer the active `bin/` generator over generated HTML.
- [ ] For active project state, use `doc/01_STATE.md`.

### 1.4 Ask for targeted mechanical checks

- [ ] Ask for `rg` checks when validating terminology or field-name drift.
- [ ] Ask for `git diff --check` after doc/code edits.
- [ ] Ask for generator regeneration when a mockup generator changes.
- [ ] Ask for crate-level tests before workspace-level tests unless shared interfaces changed.

### 1.5 Choose the right audit size

- [ ] Small change: inspect target file plus its nearest canonical spec.
- [ ] UI mockup change: inspect generator, regenerate output, search generated HTML for stale strings.
- [ ] CLI change: inspect user manual/spec command examples plus CLI code/mockup generator.
- [ ] Data format change: inspect `file-format-spec.md`, MVP schema, and tests/fixtures.
- [ ] Phase gate: do a full semantic audit across spec, CLI, mock data, diagrams, and tests.

### 1.6 Use logs as continuity, not transcript

- [ ] Ask the agent to log decisions, touched files, checks, and open risks.
- [ ] Do not require every intermediate observation to be copied into logs.
- [ ] Keep response IDs in important summaries so future sessions can find decisions.

## 2. Operating rules for Codex and other AI agents

Apply these defaults unless the user directs otherwise.

### 2.1 Default entry bundle

Read only this bundle at session start:

- `doc/01_STATE.md`
- `doc/00_AGENTS.md` sections 1-2, if canonical-file ownership matters
- the one canonical file that owns the task
- target source files named by the user or found by focused search

Do not reload the full spec stack by default.

### 2.2 Current working set policy

Before reading large files, identify the working set:

- task type
- canonical owner file
- source files to edit
- generated outputs to verify
- checks to run

If the working set is unclear, infer it from `doc/00_AGENTS.md` and
`doc/01_STATE.md`. Ask the user only when a reasonable inference would be
risky.

### 2.3 Avoid high-noise paths by default

Do not scan or summarize these paths unless directly relevant:

- `doc/04_STORYBOARDS/out/archive/`
- `doc/04_STORYBOARDS/archive/`
- `src/ui/.dart_tool/`
- `src/ui/macos/Pods/`
- `test/__pycache__/`
- `src/archive/`
- generated HTML under `doc/04_STORYBOARDS/out/`, except targeted verification

### 2.4 Prefer generators over generated HTML

For mockup work:

- edit `bin/<mockup-generator>`
- regenerate the matching `doc/04_STORYBOARDS/out/<mockup>.html`
- inspect/search generated HTML only after regeneration
- avoid treating generated HTML as source of truth

### 2.5 Search before reading

Use focused search to shrink context:

- `rg --files` to locate candidate files
- `rg -n` for terminology, flags, schema keys, commands, and old names
- line-neighborhood reads for matches
- full-file reads only for canonical docs or small files

### 2.6 Keep git and status scoped

The repo lives inside a broader workspace with unrelated dirty state.

- Prefer path-scoped checks under `prj/nyxforge`.
- Avoid broad workspace status dumps unless the task spans the workspace.
- Never revert unrelated changes.

### 2.7 Verification ladder

Use the cheapest check that catches the likely failure:

- terminology/doc edits: targeted `rg`, then `git diff --check`
- mockup edits: regenerate output, `rg` generated HTML, optionally open in browser
- Rust crate edits: `cargo test -p <crate>`
- shared Rust changes: broaden to affected crates or workspace
- shell/mock generators: execute the changed generator and inspect output

### 2.8 Logging policy

Log only durable continuity facts:

- user decisions
- files changed
- generated files regenerated
- checks run and whether they passed
- unresolved risks or follow-up tasks

Avoid logging raw command output unless it is the evidence needed for a future
agent to continue.

## 3. Recommended current working set

As of 2026-04-25, most NyxForge spec/UI/CLI sessions should begin with:

- `doc/01_STATE.md`
- `doc/00_AGENTS.md`
- `doc/00_MVP.md`, relevant sections only
- `doc/05_TECH/plugin-architecture.md`, if plugin/UI terminology is involved
- `doc/05_TECH/file-format-spec.md`, if persistent file schemas are involved
- active `bin/` mockup generators, when UI mockups are involved

Avoid by default:

- old archived storyboards
- generated HTML except targeted verification
- Flutter build artifacts
- CocoaPods vendored sources
- Python bytecode caches
- historical/superseded design docs unless doing archaeology

## 4. Phase-gate exception

At phase gates, spend the tokens. A full semantic audit should cover:

- `doc/00_MVP.md`
- relevant `doc/05_TECH/*` specs
- CLI/user manual command examples
- mock data and fixtures
- active mockup generators and regenerated outputs
- Rust types/tests for the touched surface

The goal is not to minimize tokens absolutely. The goal is to spend them where
they buy real risk reduction.
