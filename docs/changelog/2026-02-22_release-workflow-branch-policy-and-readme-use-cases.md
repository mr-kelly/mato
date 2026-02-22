# 2026-02-22 Release Workflow, Branch Policy, and README Use Cases

## Overview

This session focused on repository process hardening and product messaging updates:

1. Reworked GitHub Actions release workflow around `develop` and `main`.
2. Created and switched to `develop`; set it as default branch on GitHub.
3. Applied branch protection to both `develop` and `main`.
4. Fixed Linux `aarch64` CI linker/toolchain mismatch.
5. Refined tab naming behavior in UI (config name vs terminal title).
6. Updated README with practical high-ROI use cases and cleaned formatting.
7. Updated AGENTS policy to require one changelog per AI chat session.
8. Removed remaining Chinese text to keep repository language fully English.

---

## 1) GitHub Actions Release Workflow Changes

File: `.github/workflows/release.yml`

### Trigger strategy
- Removed tag-only trigger.
- Added:
  - `push` on `develop` and `main`
  - `workflow_dispatch` for manual trigger

### Version and tag behavior
- `develop`:
  - Uses base `Cargo.toml` version and appends `-alpha.<run_number>` during CI build.
  - Creates tag in format: `vX.Y.Z-alpha.<run_number>`.
  - Publishes prerelease.
- `main`:
  - Uses `Cargo.toml` version as-is.
  - Fails early if matching tag `vX.Y.Z` already exists on origin (forces version bump before release).

### Release flow
- Added `prepare` job to compute branch/version/tag outputs and enforce tag uniqueness.
- `build` matrix consumes computed version/tag and produces tagged artifacts.
- `release` job creates and pushes tag, then creates GitHub Release using that tag.

---

## 2) Branching and Protection Policy Applied

### Local and remote branch setup
- Created local `develop`.
- Pushed `develop` to origin with upstream tracking.

### Default branch
- GitHub default branch changed from `main` to `develop`.

### Branch protection (both `develop` and `main`)
- Enforce admin protection: enabled.
- Required PR reviews: 1 approval.
- Dismiss stale reviews: enabled.
- Required conversation resolution: enabled.
- Force pushes: disabled.
- Branch deletions: disabled.

---

## 3) CI Bug Fix: aarch64 Linux Linker Mismatch

Issue observed in GitHub Actions:
- `rust-lld` reported objects for `aarch64-unknown-linux-gnu` as incompatible with `elf64-x86-64`.

Root cause:
- Missing explicit cross-linker/toolchain wiring for aarch64 Linux target.

Fixes in workflow:
- Installed cross toolchain dependencies:
  - `gcc-aarch64-linux-gnu`
  - `binutils-aarch64-linux-gnu`
  - `libc6-dev-arm64-cross`
- Split build step:
  - `Build (aarch64 linux)` with explicit env:
    - `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc`
    - `CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc`
    - `AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar`
  - `Build (non-aarch64 targets)` for all other targets.

---

## 4) UI Naming Behavior: Config Name vs Terminal Title

Requested behavior:
- Topbar should show tab config name.
- Content panel title should show: `tab_config_name : terminal_title`.

Implementation:
- Added runtime terminal title map in app state (`tab_id -> title`).
- `sync_tab_titles()` now stores runtime title in map instead of overwriting `tab.name`.
- Terminal block title renderer now composes:
  - `tab.name` when no runtime title
  - `tab.name : runtime_title` when available

Result:
- Stable configured labels in topbar.
- Rich contextual title in content panel.

---

## 5) README Use Cases and Formatting

File: `README.md`

Updated `Perfect For` section to three practical scenarios:
- SSH/cloud host reconnect persistence with coding agents.
- Low-spec local machine + heavy AI workloads via CLI workflow.
- Single-screen monitoring of many agent terminals with activity spinners.

Also removed malformed leftover markdown/HTML around the Features section to restore clean rendering.

---

## 6) AGENTS Policy Update

File: `AGENTS.md`

Added explicit rule:
- Every AI Agent chat session must produce/maintain exactly one session changelog in `docs/changelog/`.
- A single session should continue updating the same changelog file unless explicitly requested to split.

---

## 7) Repository Language Consistency

Performed repository-wide scan for Chinese characters and translated remaining entries in `AGENTS.md` into English.

Status:
- No Chinese characters remain in tracked project files after scan.

---

## Validation Notes

Commands used during this session included:

```bash
source ~/.cargo/env && cargo test -q
source ~/.cargo/env && cargo clippy --all-targets --all-features
```

Results:
- Tests passed.
- Clippy warnings reduced significantly; only structural naming warning remained (`module_inception`).

