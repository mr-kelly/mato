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

---

## 8) Homebrew Tap Setup and Verification

### Goal
- Enable installation via:
  - `brew tap mr-kelly/tap`
  - `brew install mato`

### Actions taken
- Created repository: `mr-kelly/homebrew-tap` (public).
- Cloned to local path: `~/Documents/homebrew-tap`.
- Added formula: `Formula/mato.rb` targeting release `v0.4.0-alpha.2` assets with per-platform SHA256.
- Added tap README with install instructions.
- Committed and pushed initial tap content to `main`.
- Set default branch of `mr-kelly/homebrew-tap` to `main`.

### Root cause of previous failure
- `mr-kelly/tap` shorthand maps to `mr-kelly/homebrew-tap`.
- That repository did not exist before setup, so `brew tap` failed.

### Verification

Executed:

```bash
brew tap mr-kelly/tap
brew install mato
```

Observed result:
- Tap cloned successfully.
- Formula resolved as `mato (0.4.0-alpha.2)`.
- Installation completed successfully.

---

## 9) Remove Local Formula Template from Main Repo

### Change
- Deleted local `Formula/` directory from `mato` repository.

### Reason
- Active Homebrew formula now lives in dedicated tap repo: `mr-kelly/homebrew-tap`.
- Keeping a second formula copy in `mato` risks version drift and maintenance confusion.

### Documentation update
- Updated `docs/agents/RELEASE_GUIDE.md`:
  - Removed `cp Formula/mato.rb` flow from main repo.
  - Clarified that `Formula/mato.rb` should be updated directly in tap repo.

---

## 10) CI Automation: Update `homebrew-tap` Formula

### Goal
- Automatically update `mr-kelly/homebrew-tap/Formula/mato.rb` from release workflow.

### Workflow changes
- Added new job in `.github/workflows/release.yml`:
  - `update_homebrew_tap`
  - `needs: [prepare, release]`
  - runs only for `main` branch releases

### What the job does
1. Validates secret `HOMEBREW_TAP_TOKEN` exists.
2. Downloads `checksums.txt` from the just-created release tag.
3. Resolves per-platform asset names and SHA256 values.
4. Generates `Formula/mato.rb` content with:
   - exact release URLs
   - exact checksums
   - stable formula version
5. Clones `mr-kelly/homebrew-tap`, updates formula, commits, and pushes to `main`.

### Why main-only
- Keeps `brew install mato` stable.
- Avoids promoting `develop` alpha prereleases into default formula.

---

## 11) Dual Formula Strategy for Homebrew

### Policy update
- `main` releases update `Formula/mato.rb` (stable channel).
- `develop` releases update `Formula/mato-alpha.rb` (alpha channel).

### Workflow implementation
- Updated `.github/workflows/release.yml`:
  - `update_homebrew_tap` now runs on both `main` and `develop`.
  - Formula file/class are selected by branch:
    - `main` -> `mato.rb` / `Mato`
    - `develop` -> `mato-alpha.rb` / `MatoAlpha`
  - Added explicit `conflicts_with` declaration in generated formula content to avoid ambiguous binary collision.

### Tap repository updates
- Added `Formula/mato-alpha.rb` in `mr-kelly/homebrew-tap`.
- Added conflict declarations:
  - `mato` conflicts with `mato-alpha`
  - `mato-alpha` conflicts with `mato`

### Installation behavior
- `brew install mr-kelly/tap/mato-alpha` works, but cannot be linked simultaneously with `mato` because both install `bin/mato`.
- This is expected and now explicitly declared in formulas.

### CI test result
- `develop` release workflow reached `Update Homebrew Tap Formula` but failed at secret validation:
  - missing/empty `HOMEBREW_TAP_TOKEN` in `mr-kelly/mato` repository secrets at runtime.

---

## 12) PTY Shell Selection: Use System Default Instead of Hardcoded `bash`

### Change
- Updated `src/providers/pty_provider.rs` to resolve shell dynamically:
  - first `SHELL` env var
  - then login shell from user account (`getpwuid(...).pw_shell` on Unix)
  - fallback to `/bin/sh`

### Reason
- On macOS, users expect default shell (`/bin/zsh`) when launching terminal sessions.
- Hardcoded `bash` caused unexpected shell mismatch after Homebrew installation.

### Behavior
- New PTY sessions now open with the user's configured default shell automatically.

---

## 13) Spinner Visibility Rule: Respect Active Desk + Active Tab

### Bug
- Sidebar desk spinner was hidden whenever activity came from the desk's active tab.
- This was incorrect for non-selected desks: switching away should still show spinner for background activity.

### Fix
- Updated `src/client/ui.rs` sidebar spinner condition:
  - spinner is hidden only when the active output is from the currently visible tab
  - condition for hide: `selected desk + active tab`
  - all other active tabs (including active tab in non-selected desks) show spinner

### Result
- Current visible tab no longer shows spinner while you're watching it.
- After switching desk, background activity in the previous desk correctly shows spinner.

---

## 14) `mato --status`: Add Active Tabs + Daemon Resource Snapshot

### Change
- Enhanced `src/daemon/status.rs`:
  - query daemon `GetIdleStatus` and compute active tabs using idle threshold (`< 2s`)
  - print `Active Tabs` count alongside total tabs
  - print up to 5 active tab labels as `Office/Desk/Tab`
  - print daemon process CPU and memory snapshot (via `ps`)

### Reason
- `--status` previously showed only `Total Tabs`, but not how many tabs are currently active.
- Added lightweight runtime observability for quick health checks.

### Scope Note
- Current resource numbers are daemon-level usage (CPU/MEM), not per-tab process usage.

---

## 15) Spinner Policy Finalization + User Documentation

### Requirement refinement
- Active desk should never show spinner in sidebar.
- Active tab should never show spinner in topbar.

### Code update
- Updated `src/client/ui.rs` sidebar logic:
  - selected desk => always no spinner
  - non-selected desk => spinner if any tab is active
- Topbar rule remains:
  - active tab => no spinner
  - non-active tab => spinner when active

### Documentation
- Added `docs/SPINNER_LOGIC.md` with complete spinner behavior:
  - data source and active threshold
  - sidebar and topbar visibility rules
  - practical examples for desk/tab switching behavior
- Linked spinner doc in `README.md` documentation list.

---

## 16) `--status` Metrics Shift: from `Active Tabs` to `Running TTY Processes`

### Feedback
- `Active Tabs` was not useful for operational visibility.
- Requested metric: how many terminal processes are actually running and their resource usage.

### Change
- Added daemon protocol for process status:
  - `ClientMsg::GetProcessStatus`
  - `ServerMsg::ProcessStatus { tabs: Vec<(tab_id, pid)> }`
- Daemon now returns PTY child PID per tab (`PtyProvider::child_pid`).
- `mato --status` now reports:
  - `Running TTYs: <running>/<total tabs>`
  - `TTY Processes` section with:
    - count
    - total CPU
    - total memory
    - top entries (tab label + pid + cpu + mem)

### Result
- Status output now reflects real running terminal workload, not just recent output activity.
