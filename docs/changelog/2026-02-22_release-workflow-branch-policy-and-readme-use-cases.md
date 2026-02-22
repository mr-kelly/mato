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

---

## 12) Terminal Cursor Debugging (Claude/Gemini/Codex)

### Problem statement
Observed inconsistent cursor behavior across CLI apps in terminal tabs:
- `codex-cli`: cursor mostly correct.
- `gemini-cli`: intermittent invisible cursor, later stabilized.
- `claude code`: cursor frequently invisible or visually offset from input area.

### Key findings
- For `claude code`, runtime logs repeatedly showed:
  - `cursor_shape=Hidden`
  - `cursor_row` valid (non-zero, changing with layout)
  - `cursor_col=0` (often fixed)
- This indicates the emulator is providing cursor data, but the terminal mode semantics differ from apps that expose a visible hardware cursor.

### Implementation iterations (chronological)
1. Added explicit display width model for cells:
   - `ScreenCell.display_width` (0/1/2).
   - Wide-char spacer cells mapped to zero width.
2. Switched cursor X calculation to visual-width sum (not raw cell index).
3. Removed direct `crossterm` cursor style/show/hide calls from render path to avoid API-layer conflict with ratatui frame cursor management.
4. Added software cursor overlay (tui-term style) in UI buffer rendering.
5. Introduced low-frequency cursor debug logging for targeted tabs (`claude/gemini/codex`) capturing:
   - tab metadata
   - cursor shape/row/col
   - visible area dimensions
6. Migrated Alacritty screen extraction toward renderable content iterator flow (`display_iter`) and richer cell attributes (`dim/reverse/strikethrough/hidden`).
7. Tested multiple Hidden-cursor fallback heuristics (row shift / row-tail inference); kept conservative behavior and removed aggressive heuristics that caused right-edge drift.

### Current status at end of session
- Cross-app cursor behavior improved for `gemini-cli` and remains correct for `codex-cli`.
- `claude code` remains the hard case due to persistent `Hidden` cursor shape semantics and non-trivial input-area mapping.
- Debug instrumentation now exists to support reproducible diagnosis in subsequent sessions.

### Files touched in this debugging track
- `src/terminal_provider.rs`
- `src/emulators/alacritty_emulator.rs`
- `src/emulators/vt100_emulator.rs`
- `src/client/ui.rs`
- `src/client/app.rs`
- `tests/daemon_tests.rs`

### Notes for next session
- Keep using cursor-debug telemetry as the primary truth source before adding new heuristics.
- Prefer a single unified cursor policy over app-specific hardcoded exceptions when possible.
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

---

## 17) Multilingual Onboarding + Builder Template Cleanup

### Onboarding language switch
- Updated `src/client/onboarding_tui.rs`:
  - Added language selector with `Left/Right` switching.
  - Supported languages: English, Simplified Chinese, Traditional Chinese, Japanese, Korean.
  - Localized onboarding UI strings (title, labels, help text).
  - Localized template display content (name/description/details) per selected language.

### Template cleanup
- Updated `templates/power-user.json`:
  - Removed placeholder desks named `Desk N` (21-45).
  - Builder template reduced to meaningful desks only.
  - Updated metadata summary to `20 desks, 248 tabs`.

### Documentation updates
- Updated `templates/README.md`:
  - Renamed template section to `Mato Creator Office`.
  - Updated counts from `45/250+` to `20/248`.
  - Added onboarding language-switch note (`←/→`).
- Updated `README.md` feature line to reflect current template scale.

---

## 18) One-Template Multilingual State Application

### Decision
- Keep one canonical template structure per use case.
- Apply language localization at onboarding apply time (instead of maintaining per-language template files).

### Implementation
- Updated `src/client/onboarding_tui.rs`:
  - `apply_template_return` now receives selected language.
  - Added `localize_state_names(template_kind, language, state)` to rewrite desk/tab names before saving state.
  - Added targeted translations for:
    - Minimal template (`Desk 1`, `Terminal 1`)
    - Core desks/tabs in Solo/One-Person/Fullstack/Data templates
    - All desk names in `power-user` (Mato Creator Office)

### Result
- Onboarding language selection now affects the actual created workspace names, not only onboarding display text.
- Template structure remains single-source and consistent across all languages.

---

## 19) Move Multilingual Data out of Rust into `templates/*.json`

### Problem
- Multilingual template naming was previously hardcoded in Rust (`localize_name` style logic).
- This made copy updates and language edits code-heavy and difficult to maintain.

### Refactor
- Embedded localization directly in each template file (`templates/*.json`):
  - `metadata.name/description/details` support per-language maps (`en`, `zh-CN`, `zh-TW`, `ja`, `ko`)
  - `office/desk/tab` `name` fields also support per-language maps
- Updated `src/client/onboarding_tui.rs` to:
  - parse localized metadata directly from each template JSON
  - parse localized `office/desk/tab` names directly from each template JSON
  - localize applied workspace names from template-local data when creating state
- Removed hardcoded template-name localization logic from Rust.

### Result
- Language content now lives in template JSON data, not code.
- Adding/changing translations is now a template data edit in `templates/*.json` without Rust changes.

---

## 20) Add 3 Role-Focused Templates (Marketing / Trading / HR)

### New templates
- Added `templates/marketing-ops.json`
  - Campaign planning, social media ops, growth/SEO, CRM/email execution
  - 4 desks / 12 tabs
- Added `templates/markets-trading.json`
  - Market watch, strategy lab, execution, risk/compliance
  - 4 desks / 12 tabs
- Added `templates/hr-admin.json`
  - Recruiting, people ops, admin coordination, learning/culture
  - 4 desks / 12 tabs

### Onboarding integration
- Updated `src/client/onboarding_tui.rs` template list:
  - onboarding choices expanded from 6 to 9 templates
  - new templates are available on first run and office creation flow

### Template localization model
- Each new template includes:
  - localized `metadata` (`en`, `zh-CN`, `zh-TW`, `ja`, `ko`)
  - localized `name` objects for office/desk/tab labels

### Documentation updates
- Updated `templates/README.md`:
  - added 3 new template sections
  - updated onboarding example list to `1-9`

---

## 21) Add `Financial Trader` Persona Template + Persona Suggestions

### New template
- Added `templates/financial-trader.json`:
  - target: secondary-market stock analyst / discretionary trader
  - desks:
    - `Pre-Market`
    - `Live Market`
    - `Equity Research`
    - `Risk Book`
    - `Post-Market`
  - total: 5 desks / 15 tabs
  - includes localized metadata and localized office/desk/tab labels

### Onboarding integration
- Updated `src/client/onboarding_tui.rs`:
  - added `financial-trader` to embedded template list
  - onboarding choices expanded from `1-9` to `1-10`

### Documentation
- Updated `templates/README.md`:
  - added `Financial Trader` section
  - updated onboarding example count to `1-10`
  - added a short `Suggested Personas` section for future template expansion

---

## 22) `Financial Trader` Expanded + Full Tab Localization

### Requested adjustment
- Rename template toward `金融股票交易`.
- Add more desks for secondary-market analyst/trader workflow.
- Ensure tabs are multilingual (not just desk labels).

### Implemented changes
- Reworked `templates/financial-trader.json`:
  - template/office naming updated to `Financial Stock Trading` (`zh-CN`/`zh-TW`: `金融股票交易`)
  - expanded from 5 desks / 15 tabs to **8 desks / 24 tabs**
  - added desks:
    - `Sector Rotation`
    - `Catalyst & News`
  - all desk names and all tab names now have localized values (`en`, `zh-CN`, `zh-TW`, `ja`, `ko`)

### Documentation sync
- Updated `templates/README.md` `Financial Trader` structure and counts to match new template.

---

## 23) Remove `markets-trading` Template (Consolidate into `financial-trader`)

### Change
- Removed `templates/markets-trading.json`.
- Kept and expanded `templates/financial-trader.json` as the single finance-focused template.

### Integration update
- Updated `src/client/onboarding_tui.rs`:
  - removed `MARKETS_TRADING` include and onboarding entry
  - template selection list reduced from 10 to 9

### Documentation update
- Updated `templates/README.md`:
  - removed `Markets & Trading` section
  - re-numbered sections accordingly
  - updated onboarding example choices to `1-9`

---

## 24) Fill Remaining Multilingual Gaps for Tab/Desk/Office Names

### Problem
- Some templates still had partial localization:
  - many `tab` names only had `en`
  - some `office`/`desk` names also missed non-English keys

### Change
- Applied normalization across all `templates/*.json`:
  - ensure `metadata.name/description/details` are localized objects
  - ensure every `office/desk/tab` `name` has keys:
    - `en`, `zh-CN`, `zh-TW`, `ja`, `ko`
  - when translation was missing, fallback copied from `en`

### Result
- No missing language keys remain for template names at office/desk/tab levels.
- Templates are structurally ready for incremental translation polishing without further schema change.

---

## 25) Jump Mode Focus Routing by Origin Focus

### Requested behavior
- If Jump Mode is entered from `Content`:
  - jump to tab target -> stay in `Content`
  - jump to desk target -> stay in `Content`
- If Jump Mode is entered from `Sidebar`:
  - jump to tab target -> focus `Topbar`
  - jump to desk target -> focus `Sidebar`
- If Jump Mode is entered from `Topbar`:
  - jump to desk target -> focus `Sidebar`
  - jump to tab target -> focus `Topbar`

### Implementation
- Updated `src/client/app.rs` (`handle_jump_selection`):
  - capture `origin_focus` before applying jump
  - route final focus based on `(origin_focus, target_kind)` instead of fixed target focus

### Result
- Jump behavior now preserves flow intent for `Content` users while keeping expected cross-focus behavior between `Sidebar` and `Topbar`.

---

## 26) Update Notification Strategy Fix + Unit Tests

### Observation
- `https://mato.sh/version.txt` currently returns `0.6.0`.
- Previous logic used string inequality (`remote != current`), which could produce false positives when remote is older/different format.

### Fix
- Updated daemon update comparison in `src/daemon/daemon.rs`:
  - parse versions with SemVer (`semver` crate), including optional `v` prefix
  - notify only when `remote > current`
  - invalid/empty remote text now yields no update

### Tests
- Added unit tests for update decision function:
  - same version => no update
  - newer patch => update
  - older remote => no update
  - prerelease -> stable => update
  - stable current vs prerelease remote => no update
  - `v` prefix compatibility
  - invalid remote text => no update

---

## 27) End-to-End UpdateStatus Protocol Integration Test

### Goal
- Verify update notification data path through real Unix socket protocol handling.

### Added test
- `tests/integration_tests.rs`
  - added `start_daemon_with_latest(...)` helper to seed daemon-side `latest_version`
  - added `daemon_get_update_status_round_trip`:
    - client sends `ClientMsg::GetUpdateStatus`
    - daemon responds with `ServerMsg::UpdateStatus { latest: Some(\"0.6.1\") }`
    - test asserts exact round-trip payload

### Why this matters
- This complements unit tests of version comparison with an integration-level protocol validation.

---

## 28) App-Level End-to-End Update Refresh Integration Test

### Goal
- Verify `App::refresh_update_status()` behavior end-to-end with a real Unix socket daemon handler, not only protocol-level checks.

### Implementation
- Updated `src/client/app.rs`:
  - kept `refresh_update_status()` as default entry point
  - added `refresh_update_status_from_socket(...)` to allow socket-path injection in tests while preserving production behavior
- Added test in `tests/integration_tests.rs`:
  - `app_refresh_update_status_round_trip`
  - starts in-process daemon socket with seeded latest version (`0.6.2`)
  - sets `app.last_update_check` older than 1 hour
  - calls `app.refresh_update_status_from_socket(...)`
  - asserts `app.update_available == Some("0.6.2")`

### Validation
- `source ~/.cargo/env && cargo test -q app_refresh_update_status_round_trip --test integration_tests` passed
- `source ~/.cargo/env && cargo test -q` passed (full suite)

---

## 29) Update Check Robustness Tests (Throttle + Connection Failure)

### Goal
- Close two remaining gaps in `App` update-check behavior:
- ensure 1-hour throttle prevents unnecessary checks
- ensure daemon socket connection failure does not overwrite existing update state

### Added tests
- `tests/integration_tests.rs`
  - `app_refresh_update_status_throttled_within_one_hour`
    - daemon advertises a newer version (`9.9.9`)
    - app check is within 1-hour throttle window
    - assert `update_available` remains unchanged (`None`)
  - `app_refresh_update_status_connection_failure_does_not_overwrite_state`
    - app starts with `update_available = Some("0.6.0")`
    - socket path is missing/unreachable
    - assert previous state is preserved

### Validation
- `source ~/.cargo/env && cargo test -q --test integration_tests app_refresh_update_status_throttled_within_one_hour` passed
- `source ~/.cargo/env && cargo test -q --test integration_tests app_refresh_update_status_connection_failure_does_not_overwrite_state` passed
- `source ~/.cargo/env && cargo test -q` passed (full suite)

---

## 30) Fix Active Desk Persistence on Restart

### Report
- After exit and reopen, app did not return to the previously active desk.

### Root cause
- Desk selection mostly updated only `list_state`.
- Persisted state (`SavedOffice.active_desk`) was not updated when changing desk via navigation/click/jump.
- On next startup, restored desk could be stale.

### Fix
- Added unified desk selection API in `src/client/app.rs`:
  - `select_desk(...)` now synchronizes:
    - `list_state.selected()`
    - `offices[current_office].active_desk`
    - `dirty` when selection changes
- Replaced direct desk-selection writes to use unified API:
  - `switch_office(...)`
  - `new_desk(...)`
  - `close_desk(...)`
  - `nav(...)`
  - jump-mode desk target in `handle_jump_selection(...)`
  - sidebar mouse click in `src/main.rs`
- Startup hardening:
  - clamp restored `current_office` and `active_desk` in `App::new` and `App::from_saved`
  - fallback to default office when loaded/supplied offices are empty

### Verification
- Added persistence assertion update in `tests/daemon_tests.rs`:
  - `save_and_load_state_roundtrip` now navigates to desk index 1, saves state, and asserts restored `active_desk == 1`.

---

## 31) Fix Delayed Initial Update Banner Check

### Symptom
- Update banner could remain hidden right after startup, even when local version was intentionally lower than remote.

### Root cause
- `App` initialized `last_update_check` to `now - 290s`.
- Client throttle is 1 hour (`3600s`), so first `GetUpdateStatus` request was delayed by ~55 minutes.

### Fix
- Updated `src/client/app.rs` constructor paths (`new` and `from_saved`) to initialize:
  - `last_update_check = now - 3601s`
- This guarantees the first UI loop can check update status immediately after startup.

### Validation
- `source ~/.cargo/env && cargo test -q --test integration_tests app_refresh_update_status_round_trip` passed
- `source ~/.cargo/env && cargo test -q` passed (full suite)

---

## 32) Detect Client/Daemon Version Mismatch at Startup

### Problem
- After upgrading `mato`, an old daemon process could still be reused via existing socket.
- This leads to client/daemon version drift until manual `mato --kill`.

### Change
- `src/daemon/daemon.rs`
  - `ClientMsg::Hello` now returns daemon `CARGO_PKG_VERSION` (instead of fixed `"0.1"`).
- `src/main.rs`
  - after `ensure_daemon_running()`, client performs handshake (`Hello`) to read daemon version
  - if daemon version differs from client version, show prompt:
    - `Restart daemon now to use the new version? [Y/n]`
  - on confirm, run:
    - `mato --kill` equivalent (`daemon::kill_all()`)
    - start daemon again (`daemon::ensure_daemon_running()`)
  - if restart still reports mismatch, emit warning

### Outcome
- Upgrade flow is now safer and explicit.
- Users are prompted to recycle stale daemon process immediately when versions diverge.

---

## 33) Clarify Daemon-Restart Prompt Impact (Safer Default)

### Problem
- Version mismatch prompt asked for restart, but did not clearly explain side effects.
- Users could assume restart is non-disruptive.

### Change
- Updated `src/main.rs` restart confirmation text to explicitly state:
  - all running TTY/shell processes will be terminated
  - other running `mato` clients will be closed
  - layout/state is kept, but live process state is lost
- Changed prompt default from accept to deny:
  - from `[Y/n]` to `[y/N]`
  - empty input now means **No**

### Outcome
- User decision at upgrade time is now informed and safer by default.

---

## 34) Fix Jump-Mode Label Alignment on Topbar Tabs

### Problem
- In Jump Mode, topbar labels (`[a]`, `[b]`, ...) could appear visually misaligned with tabs.
- This was more obvious with emoji/CJK tab names.

### Root cause
- Topbar width/layout logic used byte length (`String::len`) instead of terminal display width.
- Jump label X position used a fixed offset relative to tab start.

### Fix
- Updated `src/client/ui.rs`:
  - switched tab width calculations to display width (`unicode_width::UnicodeWidthStr`)
  - adjusted Jump label rendering for topbar tabs to center inside each tab area

### Result
- Jump labels now align consistently with rendered tabs, including multilingual/emoji names.
