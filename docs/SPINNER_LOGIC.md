# Spinner Logic

This document explains how Mato decides when to show activity spinners in the UI.

## Goal

Spinners indicate background activity without distracting from the currently viewed context.

## Data Source

- Activity data comes from daemon `IdleStatus` (`tab_id -> idle_seconds`).
- Client polls daemon periodically in `App::refresh_active_status`.
- A tab is considered active when `idle_seconds < 2`.

## Spinner Rules

### Sidebar (Desk List)

- `Active desk` (currently selected desk) never shows a spinner.
- Any non-selected desk shows a spinner when at least one of its tabs is active.

This keeps the current desk visually stable while still surfacing background work elsewhere.

### Topbar (Tab Bar)

- `Active tab` (currently selected tab) never shows a spinner.
- Any non-selected tab in the current desk shows a spinner if active.

This lets you see which sibling tabs in the same desk are producing output.

## Practical Examples

1. You are on `Desk A / Tab C`, and `Tab C` is busy:
- Desk A: no spinner
- Tab C: no spinner

2. You are on `Desk A / Tab C`, and `Tab D` is busy:
- Desk A: no spinner
- Tab D: spinner

3. You switch to `Desk B`, while `Desk A / Tab C` is still busy:
- Desk A: spinner
- Desk B (active desk): no spinner

## Notes

- Spinner is a recency indicator (recent output), not CPU usage.
- `mato --status` includes active tab count and daemon CPU/memory snapshot for runtime visibility.
