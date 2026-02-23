# Mato v0.8.1 Release Summary

## Theme

Jump Mode correctness in visible regions + startup/render stability polish.

## Major Changes

### Jump Mode Labeling and Routing

- Expanded jump label set to include `0-9`.
- Added focus-aware reserved key filtering:
  - Content reserves `c/r/q`
  - Sidebar and Topbar reserve `r/q`
- Updated jump dispatch to accept alphanumeric keys.

### Viewport-Aware Jump Targets

- Sidebar jump targets now use list viewport (`offset` + visible rows).
- Sidebar jump label rendering now uses local row mapping (`index - offset`) to keep overlay aligned after scrolling.
- Topbar jump targeting remains based on visible tab index mapping.

### Incremental Screen Update Path

- Integrated `ScreenDiff` incremental update behavior and tests into release line.
- Small updates can use changed-line payloads instead of full-frame updates.

### Runtime Stability and Observability

- Startup ordering and rendering alignment adjusted to reduce first-frame instability.
- High-frequency lifecycle logs lowered from info to debug to reduce runtime noise.
- Daemon module source path consolidated to `src/daemon/service.rs`.

## Verification

Validation command used for release prep:

```bash
source ~/.cargo/env && cargo build -q && cargo test -q
```
