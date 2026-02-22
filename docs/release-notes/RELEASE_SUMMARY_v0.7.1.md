# Mato v0.7.1 Release Summary

## Theme

Onboarding correctness and terminal transition cleanup.

## Major Changes

### Onboarding State Semantics

- Added mode-specific onboarding behavior:
  - first-run onboarding (startup)
  - in-app onboarding (`New Office`)
- First-run and in-app key handling are now intentionally different (`q` vs `Esc` semantics).
- Updated template order priority:
  - `Start from Scratch` first
  - `Mato Creator Office` second
- Expanded `Start from Scratch` baseline to 3 desks with 2 tabs each.

### Runtime Screen Ownership

- Runtime onboarding is now integrated into main loop screen routing (`ScreenState`).
- Removed nested runtime onboarding terminal ownership path, reducing transition fragility.

### Terminal Residue Mitigation

- Strengthened clear/reposition cleanup around alt-screen transitions.
- Startup onboarding now clears immediately before first render to avoid shell ghosting.

### Code Quality

- Simplified conditional style logic in `src/client/ui.rs`.
- Replaced manual `% 2 == 0` with `.is_multiple_of(2)` for spinner status.
- Added `Default` for `TerminalGuard`.
- Removed unnecessary cast in `src/emulators/alacritty_emulator.rs`.

## Verification

Validation command:

```bash
source ~/.cargo/env && cargo fmt && cargo build -q && cargo test -q
```

Result: passing in this release prep session.
