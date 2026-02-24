# 2026-02-24 - TODOs Roadmap Audit

## Scope

- Re-audited `docs/todos/roadmap.md` against current codebase state.
- Updated stale status text in `docs/todos/README.md`.

## Changes

1. Updated roadmap metadata baseline from `v0.7.1` to `v0.9.5`.
2. Marked several capabilities as partial where implementation exists but is incomplete:
   - CPU control (adaptive poll + conditional render)
   - Per-tab process config (`cwd` works; `env/argv/custom shell` still missing)
   - Focus-aware refresh throttling
   - Resize strategy (`fixed`/`sync` tradeoff model)
   - High-throughput output policy
   - mosh-specific handling
3. Updated execution plan language to reflect implemented features:
   - Resize strategy now documented as existing and requiring formalization/tests.
   - Graphics passthrough moved from "implement" to "hardening/compatibility".
4. Updated ready-to-assign cards:
   - `resize-mode` -> `resize-mode-docs`
   - `graphics-passthrough` -> `graphics-compat`
5. Updated `docs/todos/README.md` current status line to avoid outdated `v0.2.0` reference.

## Notes

- `docs/todos/TODO.md` remains legacy and significantly behind current versions; `roadmap.md` should be treated as the authoritative TODO/audit document until TODO is refreshed.

## Follow-up (same session)

- README cleanup: removed the inline test-suite table/details from `README.md`.
- Added dedicated testing doc: `docs/TESTING.md` with standard test and snapshot commands.
- Linked testing docs from README Resources and Test Suite section.
- Replaced `docs/todos/TODO.md` with a concise execution backlog synced to `docs/todos/roadmap.md` priorities (P0-P4 + task cards).
- Updated `docs/todos/README.md` so TODO is no longer marked as legacy.
