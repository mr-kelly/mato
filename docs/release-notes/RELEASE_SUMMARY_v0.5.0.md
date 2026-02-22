# Mato v0.5.0 Release Summary

## Theme

Interaction consistency and scale-readiness, with emphasis on Jump Mode behavior, onboarding ergonomics, and theme clarity.

## Major Changes

### Jump Mode

- Added 52-label support (`a-z/A-Z`).
- Reworked target allocation strategy to avoid one-side starvation in large templates.
- Unified visual labels and key mapping through a single target-generation path.
- Removed `w/a` focus shortcuts to prevent collision with alphabetic jump labels.
- Implemented explicit directional focus matrix across focus contexts.

### Onboarding

- Added identity-based default office naming (`username` / `hostname` fallback chain).
- Added explicit rename mode in onboarding (`r`), while keeping direct Enter-to-start path.
- Applied selected/prepared office name to template state at onboarding apply time.

### Template Naming

- Renamed Power User template branding to "Mato Creator Office" in metadata and onboarding display text.

### Rendering and UI

- Improved system-theme visibility for active focus/selection states.
- Fixed Jump popup visual bleed-through with clearer popup rendering behavior.
- Fixed sidebar jump-label row alignment.
- Reworked top-right daemon status indicator semantics (`✓`, `·`, `•`).

## Documentation Updates

- Added release notes and summary for v0.5.0.
- Added/updated changelog documentation for 2026-02-22 polish cycle.
- Added operations quick guide and refreshed keyboard shortcuts reference.

## Validation

```bash
source ~/.cargo/env && cargo test -q
```

Result: passing.
