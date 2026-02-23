<div align="center">

<img src="logo.svg" alt="Mato Logo" width="160">

# Mato

### 🏖️ The Multi-Agent Terminal Office
**Managing hundreds of AI agents from your terminal.**

English: MAY-to, Spanish: undefined  
粤语: 咩圖, 한국어: 메이토, 日本語: メイト 

[![Rust](https://img.shields.io/badge/rust-stable-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue?style=for-the-badge)](LICENSE)
[![Version](https://img.shields.io/github/v/release/mr-kelly/mato?display_name=tag&style=for-the-badge)](https://github.com/mr-kelly/mato/releases/latest)
[![Stars](https://img.shields.io/github/stars/mr-kelly/mato?style=for-the-badge&color=gold)](https://github.com/mr-kelly/mato/stargazers)

[**Quick Start**](#quick-start) • [**Features**](#features) • [**Why Mato?**](#why-mato) • [**Development**](#development) • [**Resources**](#resources)

<img src="docs/images/screenshot-coding.png" alt="Mato Screenshot" width="900" style="border-radius: 12px; border: 1px solid #1C1C2A;">

---

**Mato** is a terminal multiplexer and workspace that brings visual intelligence to the CLI.

It turns your terminal into an "Office" with Desks and Tabs, where you can monitor parallel tasks, manage complex agent swarms, and keep long-lived background sessions organized — all without keyboard conflicts.

</div>

---

<a id="quick-start"></a>
## 🚀 Get Started in 60s

### 1. Install Manually
```bash
# Quick Install (Linux/macOS)
curl -fsSL http://mato.sh/install.sh | bash
```

```bash
# Homebrew (Linux/macOS)
brew tap mr-kelly/tap
brew install mato
```

```bash
# Manual Installation
tar xzf mato-*.tar.gz
sudo mv mato /usr/local/bin/
```

### 2. Install with an AI Agent

Copy this prompt into **Claude Code / Codex / Warp / Cursor / Antigravity / OpenClaw / GitHub Copilot (VS Code) / Gemini CLI / Windsurf**:

```text
Install Mato on this machine and verify it works.

1) Run: curl -fsSL http://mato.sh/install.sh | bash
2) If it fails, fallback to Homebrew:
   - brew tap mr-kelly/tap
   - brew install mato
3) Verify with:
   - mato --version
   - start mato once, then exit
4) If all install paths fail, continue with GitHub release binary, then source build.
```

### 3. Launch your first Office
```bash
mato
```
Select a template (we recommend **Full-Stack** for your first run) and start coding.

### 4. Mastering the Flow
*   **`Esc`**: Enter Jump Mode (Teleport anywhere)
*   **`n`**: New Desk/Tab (Context-aware)
*   **`r`**: Rename instantly
*   **`Enter`**: Dive back into the terminal

---

<a id="why-mato"></a>
## 👁️ The Vision: Visual Intelligence for CLI

Traditional terminal multiplexers (tmux/screen) are "blind." You never know what's happening in another tab until you switch to it. **Mato changes that.**

| **Pain** | **Mato Solution** |
| :--- | :--- |
| 🕵️ **Lost in Tabs** | See active agents instantly with live activity signals across desks/tabs. |
| 🎯 **Arrow-Key Grind** | Jump to what you need in one move, instead of stepping tab-by-tab. |
| ⌨️ **Shortcut Hell** | Only `Esc` is special, so your shell/editor shortcuts keep working as usual. |
| 🔌 **Session Loss** | Agents keep running in the background; reconnect and continue where you left off. |

---

<a id="features"></a>
## ✨ Features

<table border="0">
<tr>
<td width="50%" valign="top">

<img src="docs/images/screenshot-office.png" alt="Offices, Desks, and Tabs workspace layout in Mato" width="100%" style="aspect-ratio: 16 / 10; object-fit: cover; border-radius: 10px;">

**1) Offices -> Desks -> Tabs UI**

Organize complex work into a clear hierarchy: one Office, multiple Desks, and Tabs inside each Desk for parallel tasks.

</td>
<td width="50%" valign="top">

<img src="docs/images/screenshot-working-spinner.gif" alt="Real-time spinner indicators showing active AI tasks in tabs and desks" width="100%" style="aspect-ratio: 16 / 10; object-fit: contain; background: #0b0f14; border-radius: 10px;">

**2) Spinner Activity (AI is working)**

See where work is happening right now: active tabs and desks show live spinners when an agent or process prints output.

</td>
</tr>
<tr>
<td width="50%" valign="top">

<img src="docs/images/screenshot-multi-client-sync.gif" alt="Two SSH clients attached to the same Mato daemon showing synchronized workspace state" width="100%" style="aspect-ratio: 16 / 10; object-fit: cover; border-radius: 10px;">

**3) Multi-Client Sync**

Attach from multiple terminals/SSH sessions and keep everyone on the same live workspace state.

</td>
<td width="50%" valign="top">

<img src="docs/images/screenshot-onboarding.png" alt="Onboarding template selector with prebuilt office templates" width="100%" style="aspect-ratio: 16 / 10; object-fit: cover; border-radius: 10px;">

**4) Prebuilt Templates**

Start in seconds with ready-made templates (Mato Creator Office, Full-Stack, Solo Dev) and multilingual onboarding (English, 中文, 日本語, 한국어).

</td>
</tr>
<tr>
<td width="50%" valign="top">

<img src="docs/images/screenshot-daemon-background-run.gif" alt="AI agents continue running in the background after terminal or SSH disconnect" width="100%" style="aspect-ratio: 16 / 10; object-fit: cover; border-radius: 10px;">

**5) Background Runtime Persistence**

Your agents keep running even if the terminal closes or SSH drops. Reopen `mato` and you land back in the same live workspace.

</td>
<td width="50%" valign="top">

<img src="docs/images/screenshot-mouse-jump.gif" alt="Mouse interaction in TUI: clicking desks and tabs directly" width="100%" style="aspect-ratio: 16 / 10; object-fit: cover; border-radius: 10px;">

**6) Mouse Support in a TUI**

Click desks and tabs directly when you want speed, while keeping a terminal-native workflow.

</td>
</tr>
<tr>
<td width="50%" valign="top">

<img src="docs/images/screenshot-quick-jump-mode.gif" alt="Jump Mode quick navigation across sidebar and topbar targets" width="100%" style="aspect-ratio: 16 / 10; object-fit: cover; border-radius: 10px;">

**7) Jump Mode (Core Navigation)**

Press `Esc` to enter Jump Mode, then jump straight to visible targets without repeated arrow-key navigation.

</td>
<td width="50%" valign="top">

<img src="docs/images/screenshot-themes.png" alt="Customizable themes and persistent Office/Desk/Tab state in Mato" width="100%" style="aspect-ratio: 16 / 10; object-fit: cover; border-radius: 10px;">

**8) Customizable Themes**

Pick the theme style you like, while Office/Desk/Tab state stays persistent across reconnects and restarts.

</td>
</tr>
</table>

---

## ⌨️ Shortcut Philosophy

Mato follows the **"Rule of One"**: you don't need to memorize shortcuts. `Esc` is the only state-switch key, and everything else stays with your shell.
By default, Mato does not intercept your normal `Ctrl-*` and `Alt-*` shell/editor shortcuts.

| Key | Action | Context |
| :--- | :--- | :--- |
| **`Esc`** | **Switch State (Jump / Back)** | Global |
| `n` | Create New | Sidebar/Topbar |
| `x` | Close / Terminate | Sidebar/Topbar |
| `r` | Rename | Sidebar/Topbar |
| `o` | Office Selector | Sidebar |
| `q` | Soft Quit | Sidebar |

---

<a id="development"></a>
## 👩‍💻 Development with Coding Agents

If you want to improve a feature, fix a bug, or change behavior you are not satisfied with, paste the prompt below into your coding agent and let it implement the change for you.

Recommended agents: **Claude Code / Codex / Warp / Cursor / Antigravity / OpenClaw / GitHub Copilot (VS Code) / Gemini CLI / Windsurf**.

### 🧪 Test Suite

Mato has **101 passing tests** across 14 test suites:

| Suite | Tests | What it covers |
|---|---|---|
| `screen_diff_tests` | 16 | ScreenDiff protocol, bell/focus propagation, msgpack roundtrip |
| `input_tests` | 24 | Key bindings, jump mode, rename, copy mode, alt/ctrl encoding |
| `app_tests` | 29 | Desk/tab lifecycle, nav, rename, `from_saved` clamping, focus events, mouse cache, spinner |
| `daemon_tests` | 20 | Alacritty emulator (bell, focus tracking, wide chars, resize), PID/lock files, persistence |
| `ui_snapshot_tests` | 6 | TUI layout regressions (ratatui `TestBackend` + insta snapshots) |
| `protocol_tests` | 11 | ClientMsg/ServerMsg JSON + msgpack serde, `ScreenContent` roundtrips |
| `terminal_persistence_tests` | 5 | PTY content survives reconnect, resize, multi-write |
| `utils_tests` | 5 | ID uniqueness under load, path layout |
| others | 5 | Config, compat, integration |

```bash
cargo test                     # run all tests
INSTA_UPDATE=always cargo test --test ui_snapshot_tests  # regenerate snapshots
cargo insta review             # review changed snapshots interactively
```

### Fast Contribute (3 steps)

1. Clone and create a branch:
```bash
git clone https://github.com/mr-kelly/mato.git
cd mato
git switch -c feat/your-change
```
2. Make changes and validate:
```bash
cargo build
cargo test
```
3. Commit with Conventional Commits, push, and open a PR to `mr-kelly/mato`.

```text
GitHub project: https://github.com/mr-kelly/mato

I want to contribute a change to Mato:
[Describe your bug report or feature request in detail]

Please do the following:
1) Clone the repository and create a feature branch from latest develop (use the project’s current contribution flow).
2) Read AGENTS.md and CHANGELOG-related docs first, then reproduce the issue (or clarify expected behavior for the feature).
3) Implement a minimal, production-safe fix.
4) Run checks and tests:
   - cargo build
   - cargo test
5) Update docs affected by this change, following AGENTS.md documentation standards (including changelog/release-notes conventions where applicable).
6) Create commit(s) using Conventional Commits format (e.g., fix:, feat:, docs:, refactor:).
7) Push the branch and open a Pull Request to the original Mato repository.
8) In the PR description, clearly explain:
   - root cause summary
   - files changed
   - test/check results
   - what changed and why
   - changelog/docs updates completed
9) Do NOT create a GitHub Issue for this task.
```

Tip: the better your `[Describe ...]` section (expected behavior, actual behavior, logs, screenshots), the better and faster the result.

---

### 🍅 Pronunciation

You say tomato, I say Mato.

- English: MAY-to  
- 粤语: 咩圖  
- 普通话: 番茄终端 
- 日本語: メイト  
- 한국어: 메이토  
- Spanish: `...maybe don't translate this one.`

---

<a id="resources"></a>
## 🛠️ Resources

### 📖 Documentation
- [**Keyboard Shortcuts**](docs/KEYBOARD_SHORTCUTS.md)
- [**AI Agent Guide**](docs/AI_AGENT_FRIENDLY.md)
- [**Persistence Specs**](docs/TERMINAL_PERSISTENCE.md)
- [**Spinner Logic**](docs/SPINNER_LOGIC.md)

### 🔧 Customization
- [**Theme Engine**](docs/changelog/2026-02-22_themes-settings-update-check.md)
- [**Template Gallery**](templates/README.md)
- [**Configuration API**](src/config.rs)

---

<div align="center">

### Built for the future of development.
Join the **Mato** community and stop hunting for active terminals.

[**Star this project**](https://github.com/mr-kelly/mato) • [**Join Discord Community**](https://discord.gg/KFVUD7q2Zn) • [**Report a Bug / Request a Feature**](https://github.com/mr-kelly/mato/issues) • [**Follow Roadmap**](docs/todos/roadmap.md)

**Made with 🏖️ for developers who value clarity.**

</div>
