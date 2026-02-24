# Mato v0.9.6 Release Summary

## Theme

Compatibility + test reliability + docs consistency.

## Major Changes

### Runtime/Release

- Linux release workflow runners changed from `ubuntu-latest` to `ubuntu-20.04` for Linux targets.

### Installer UX

- `install.sh` prints shell-aware PATH/source instructions based on active shell.

### Test Reliability

- Stabilized UI snapshot behavior under differing `COLORTERM` environments.
- Refactored truecolor-related tests to deterministic value-based assertions.

### Documentation

- Added dedicated testing doc (`docs/TESTING.md`).
- Kept README lean by linking tests docs instead of embedding suite tables.
- Refreshed TODO execution docs and roadmap alignment.

## Verification

```bash
source ~/.cargo/env && cargo test
```
