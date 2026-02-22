# Mato v0.6.0 Release Summary

## Theme

Terminal resilience in day-to-day multi-desk operation, plus public website maturity.

## Major Changes

### PTY and Daemon Reliability

- Introduced PTY child-process liveness checks and automatic respawn for exited shells.
- Converted fragile spawn paths from panic-on-error to recoverable retry behavior.
- Added fallback shell retry (`/bin/sh`) when primary shell spawn fails.
- Daemon now ensures PTY availability before serving critical client actions.

### Desk Switching Behavior

- Sidebar desk navigation now proactively spawns destination active tab.
- Jump-mode desk jump also proactively spawns destination active tab.
- Client first-frame fallback for missing tab now performs synchronous spawn and immediate fetch retry.

### Cursor/Render Path

- Replaced title-sync `get_screen(1,1)` probe with current terminal size fetch.
- Added wide-char spacer-cell handling to avoid width drift and cursor mismatch in complex CLIs.

### Website

- Reworked landing page structure and visual system to reflect README positioning.
- Added asset sync for logo and screenshot in website build pipeline.
- Unified website runtime scripts around `pnpm` execution chain.

## Tests and Validation

Added/updated regression coverage:

- `sync_tab_titles_uses_current_terminal_size`
- `nav_between_desks_spawns_target_active_tab`

Validation command:

```bash
source ~/.cargo/env && cargo test -q
```

Result: passing.
