# 2026-02-22 — Shell Experience, Jump Mode, and System Theme Polish

## Summary

This iteration focused on practical UX and performance polish for real-world heavy usage (Power User template), with special attention to typing smoothness, Jump Mode consistency, and theme behavior in `system` mode.

## Shell Experience and Rendering Stability

### Typing flicker mitigation

- Removed transient blank-frame behavior during fast typing by preserving last valid screen cache instead of aggressively invalidating on every input write.
- Added fallback to cached content when synchronous fetch fails temporarily.
- Reduced render-side control noise by only sending cursor-style escape sequence when shape changes.

### Screen fetch load reduction

- Improved daemon-provider fetch behavior toward lower overhead operation under many tabs.
- Shifted expensive repeated round-trips to smoother cache/worker behavior and lower-noise logging paths.

## Daemon and Log Hygiene

- Reclassified expected disconnect-style socket errors (broken pipe family) away from noisy error reporting.
- Lowered severity for self-healing `tab not found` flows used during tab/provider recovery.
- Aligned no-response message handling for fire-and-forget style client operations.

## Theme and Visual Behavior

### New `system` default theme

- Added `system` theme as default when no explicit theme is selected.
- `system` uses terminal/OS colors (`Color::Reset`) for general UI surfaces.
- Settings UI now includes `system (follow terminal)` option.

### Terminal area and shell color matching

- Removed hardcoded terminal black fill in content panel path.
- Terminal panel background now follows active theme behavior.
- For non-system themes, added ANSI named color mapping derived from theme palette so shell content better matches selected UI theme.

### System mode clarity improvements

- Increased active focus readability in system mode:
  - active panel thick border
  - stronger active title styling
  - clearer tab/sidebar selected-state emphasis
- Fixed theme mismatch for the left-bottom focus badge (`Focus: Sidebar/Topbar/Content`) so non-system themes use theme colors instead of fixed reverse style.

## Status Indicators

- Replaced top-right `⚡` indicator behavior with simpler state semantics:
  - connected: `✓`
  - connecting/in-progress: animated `·` / `•`

## Jump Mode UX and Consistency

### Entry behavior

- Unified `Esc` behavior: entering Jump Mode from all focus contexts (`Sidebar`, `Topbar`, `Content`).
- Status bar updated to reflect `Esc -> Jump` consistently.

### Focus after selection

- Standardized jump outcome:
  - selecting a **desk** target focuses `Sidebar`
  - selecting a **tab** target focuses `Topbar`

### Label alignment and visibility

- Fixed sidebar jump label vertical offset to align with desk rows inside bordered list content.
- Fixed Jump Mode popup transparency bleed by clearing and drawing a solid popup background.

### Label capacity and mapping

- Extended jump labels from lowercase-only to `a-z + A-Z` (52 targets).
- Enabled uppercase input handling for jump.
- Removed `w/a` jump-mode focus shortcuts to avoid collisions with alphabetic jump labels.
- Ensured visual label order and key mapping use one unified target-generation path.

### Power User template behavior

- Addressed large-workspace limitation where tabs could lose labels.
- Updated target allocation strategy:
  - `Content`: balanced interleaving between tab and desk targets
  - `Topbar`: tabs first, then desks
  - `Sidebar`: desks first, then tabs
- Matched tab jump targets to visible tab labels (scrolled topbar safety).

## Documentation and Release Flow

- Prepared v0.4.0 release documentation set and changelog updates.
- Added v0.4.0 release notes and technical summary files.

## Validation

- Repeated full test runs passed after each behavior cluster update:

```bash
source ~/.cargo/env && cargo test -q
```

