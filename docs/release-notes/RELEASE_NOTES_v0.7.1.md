# Release Notes - Mato v0.7.1

**Release Date**: 2026-02-22  
**Status**: Stable

## Overview

v0.7.1 is a behavior-correctness release focused on onboarding flow safety and terminal cleanup consistency. It resolves edge cases around first-run onboarding exit behavior, removes screen residue during transitions, and simplifies runtime onboarding ownership in the main UI loop.

## Highlights

- Runtime onboarding now runs under the main screen state loop (single terminal owner).
- First-run onboarding and in-app onboarding now use explicit mode-specific key behavior.
- Startup and exit cleanup paths were hardened to reduce terminal residue.
- Small code quality improvements landed in UI and terminal utility paths.

## What's New

### 1. Onboarding Mode Split

Onboarding behavior is now explicit by context:

- **First Run** (no `state.json`): `q` quits setup flow and exits cleanly.
- **New Office** (in-app): `Esc` returns to the main UI.

This removes ambiguous cancel semantics between setup and in-app creation paths.

### 1.1 Onboarding Template Prioritization

Template ordering was adjusted to favor common bootstrap workflows:

- `Start from Scratch` is now first.
- `Mato Creator Office` is now second.
- `Start from Scratch` now initializes with 3 desks and 2 tabs per desk.

### 2. Single-Loop Runtime Onboarding

Runtime onboarding was moved into the main `ScreenState` routing instead of managing a separate nested terminal loop. This improves predictability of rendering and input handling.

### 3. Terminal Cleanup Hardening

Cleanup now enforces explicit clear/reposition behavior before leaving alt-screen where applicable, reducing shell/TUI residue and transition artifacts.

### 4. UX and Code Hygiene

- Onboarding help text now reflects actual mode behavior (`q Quit` vs `Esc Back`).
- UI style branch logic simplified in sidebar/topbar rendering.
- `TerminalGuard` now implements `Default`.
- Minor emulator cleanup: removed an unnecessary cast.

## Upgrade Notes

Recommended restart flow:

```bash
mato --kill
mato
```

## Related Docs

- [CHANGELOG.md](../../CHANGELOG.md)
- [RELEASE_SUMMARY_v0.7.1.md](RELEASE_SUMMARY_v0.7.1.md)
- [2026-02-22_terminal-rendering-overhaul-and-bell-forwarding.md](../changelog/2026-02-22_terminal-rendering-overhaul-and-bell-forwarding.md)
