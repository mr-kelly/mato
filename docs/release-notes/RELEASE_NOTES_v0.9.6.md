# Release Notes - Mato v0.9.6

**Release Date**: 2026-02-24  
**Status**: Stable

## Overview

v0.9.6 focuses on Linux installation/runtime compatibility and test reliability.

## Highlights

- Linux release build baseline changed to `ubuntu-20.04` for broader glibc compatibility.
- Installer shell guidance is now shell-aware (`bash`/`zsh`/`fish`) instead of hardcoded `.bashrc` instructions.
- Test stability improvements for environment-sensitive theme/snapshot behavior.
- Docs and TODO backlog alignment continued from v0.9.5 cleanup.

## What's New

### 1. Linux Binary Compatibility

Release workflow now builds Linux artifacts on `ubuntu-20.04`, reducing risk of runtime errors like:

```text
/lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found
```

### 2. Installer Shell Detection

`install.sh` now detects shell type and prints more accurate PATH/source instructions.

### 3. Test Determinism

- Snapshot tests no longer drift based on local `COLORTERM` environment differences.
- Truecolor tests now use deterministic parser checks instead of global env mutation.

## Validation

```bash
source ~/.cargo/env && cargo test
```

## Related Docs

- [CHANGELOG.md](../../CHANGELOG.md)
- [RELEASE_SUMMARY_v0.9.6.md](RELEASE_SUMMARY_v0.9.6.md)
- [docs/TESTING.md](../TESTING.md)
