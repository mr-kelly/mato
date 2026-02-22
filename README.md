<div align="center">

<img src="logo.svg" alt="Mato Logo" width="160">

# Mato

### 🏖️ The Multi-Agent Terminal Office
**Managing hundreds of AI agents from the command line.**

English: MAH-toh /ˈmɑːtoʊ/, 普通话: 吗头（mǎ tóu）  
粤语: 嗎桃（maa1 tou4）, 한국어: 마토（ma-to）, 日本語: マト（mato）

[![Rust](https://img.shields.io/badge/rust-stable-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue?style=for-the-badge)](LICENSE)
[![Version](https://img.shields.io/github/v/release/mr-kelly/mato?display_name=tag&style=for-the-badge)](https://github.com/mr-kelly/mato/releases/latest)
[![Stars](https://img.shields.io/github/stars/mr-kelly/mato?style=for-the-badge&color=gold)](https://github.com/mr-kelly/mato/stargazers)

[**Quick Start**](#-get-started-in-60s) • [**Showcase**](#-showcase) • [**Why Mato?**](#-the-vision) • [**Documentation**](#-pro-resources)

<img src="docs/images/screenshot-0.png" alt="Mato Screenshot" width="900" style="border-radius: 12px; border: 1px solid #1C1C2A;">

---

**Mato** is a terminal multiplexer and workspace that brings visual intelligence to the CLI.

It turns your terminal into an "Office" with Desks and Tabs, where you can monitor parallel tasks, manage complex agent swarms, and keep long-lived background sessions organized — all without keyboard conflicts.

</div>

---

## 🚀 Get Started in 60s

### 1. Installation
```bash
# Quick Install (Linux/macOS)
curl -fsSL https://raw.githubusercontent.com/mr-kelly/mato/main/install.sh | bash
```

```bash
# Homebrew (macOS/Linux)
brew tap mr-kelly/tap
brew install mato
```

```bash
# Manual Installation
tar xzf mato-*.tar.gz
sudo mv mato /usr/local/bin/
```

### 2. Launch your first Office
```bash
mato
```
Select a template (we recommend **Full-Stack** for your first run) and start coding.

### 3. Mastering the Flow
*   **`Esc`**: Enter Jump Mode (Teleport anywhere)
*   **`n`**: New Desk/Tab (Context-aware)
*   **`r`**: Rename instantly
*   **`Enter`**: Dive back into the terminal

---

## 👁️ The Vision: Visual Intelligence for CLI

Traditional terminal multiplexers (tmux/screen) are "blind." You never know what's happening in another tab until you switch to it. **Mato changes that.**

| **The Problem** | **The Mato Solution** |
| :--- | :--- |
| 🕵️ **Lost in Tabs** | **Real-time Activity Spinners** notify you exactly where the work is happening. |
| ⌨️ **Shortcut Hell** | **Zero-Conflict Design**: Your shell belongs to you. Only `Esc` is special. |
| 📉 **Task Anxiety** | **Visual Breadcrumbs**: Instant status of every background agent or build process. |
| 🔌 **Session Loss** | **Daemon-First Architecture**: Your workspace lives even if the client dies. |

---

## ✨ Premium Features

<table border="0">
<tr>
<td width="50%" valign="top">

### 🎯 Live Activity Monitoring
Never poll your terminals again. **Mato's signature spinners** appear in your sidebar and topbar the moment a process produces output. Perfect for tracking long-running builds or AI agents.

### 🤖 AI-Agent Native
Built specifically for tools like **Claude Code, Cursor, and Windsurf**. Mato preserves 100% of your shell's keyboard shortcuts, ensuring your agents operate without interference.

</td>
<td width="50%" valign="top">

### ⚡ Jump Mode (EasyMotion)
Navigate like a pro. Hit `Esc` and use **EasyMotion-style jump labels** to teleport to any desk or tab instantly. No more repetitive arrow-key mashing.

### 🍱 Office Templates
Start with the perfect setup. Choose from 6 curated templates:
- **Mato Creator Office**: 20 desks / 248 tabs for complex swarms.
- **Full-Stack**: Frontend, Backend, and DB in one view.
- **Solo Dev**: Focused, minimalist productivity.

</td>
</tr>
</table>

---

## 📽️ Showcase

<table border="0">
<tr>
<td width="50%" valign="top">

<img src="docs/images/screenshot-0.png" alt="Offices, Desks, and Tabs workspace layout in Mato" width="100%">

**1) Offices -> Desks -> Tabs UI**

Image description: A structured workspace hierarchy where Offices contain multiple Desks, and each Desk contains Tabs for parallel terminal workflows.

</td>
<td width="50%" valign="top">

<img src="docs/images/screenshot-0.png" alt="Real-time spinner indicators showing active AI tasks in tabs and desks" width="100%">

**2) Spinner Activity (AI is working)**

Image description: Live spinner indicators appear on active tabs/desks when an AI agent or process produces output, so you can see progress at a glance.

</td>
</tr>
<tr>
<td width="50%" valign="top">

<img src="docs/images/screenshot-0.png" alt="Two SSH clients attached to the same Mato daemon showing synchronized workspace state" width="100%">

**3) Multi-Client Sync**

Image description: Two SSH clients attach to the same running Mato daemon and view the same workspace state in sync.

</td>
<td width="50%" valign="top">

<img src="docs/images/screenshot-0.png" alt="Onboarding template selector with prebuilt office templates" width="100%">

**4) Prebuilt Templates**

Image description: Fast onboarding with prebuilt templates for different workflows, so teams can start from a sensible office layout.

</td>
</tr>
<tr>
<td width="50%" valign="top">

<img src="docs/images/screenshot-0.png" alt="AI agents continue running in the background after terminal or SSH disconnect" width="100%">

**5) Background Runtime Persistence**

Image description: AI agents keep running in the background even if your terminal process exits or SSH disconnects; when you open `mato` again, you return to the same live workspace automatically (no explicit detach/attach workflow).

</td>
<td width="50%" valign="top">

<img src="docs/images/screenshot-0.png" alt="Mouse interaction in TUI: clicking desks and tabs directly" width="100%">

**6) Mouse Support in a TUI**

Image description: Native mouse interaction for clicking tabs and desks; terminal-first UX with practical GUI-like navigation support.

</td>
</tr>
</table>

---

## ⌨️ Shortcut Philosophy

Mato follows the **"Rule of One"**: you don't need to memorize shortcuts. `Esc` is the only state-switch key, and everything else stays with your shell.

| Key | Action | Context |
| :--- | :--- | :--- |
| **`Esc`** | **Switch State (Jump / Back)** | Global |
| `n` | Create New | Sidebar/Topbar |
| `x` | Close / Terminate | Sidebar/Topbar |
| `r` | Rename | Sidebar/Topbar |
| `o` | Office Selector | Sidebar |
| `q` | Soft Quit | Sidebar |

---

## 🛠️ Pro Resources

<table border="0">
<tr>
<td>

#### 📖 Documentation
- [**Keyboard Shortcuts**](docs/KEYBOARD_SHORTCUTS.md)
- [**AI Agent Guide**](docs/AI_AGENT_FRIENDLY.md)
- [**Persistence Specs**](docs/TERMINAL_PERSISTENCE.md)
- [**Spinner Logic**](docs/SPINNER_LOGIC.md)

</td>
<td>

#### 🔧 Customization
- [**Theme Engine**](docs/changelog/2026-02-22_themes-settings-update-check.md)
- [**Template Gallery**](templates/README.md)
- [**Configuration API**](src/config.rs)

</td>
</tr>
</table>

---

<div align="center">

### Built for the future of development.
Join the **Mato** community and stop hunting for active terminals.

[**Star this project**](https://github.com/mr-kelly/mato) • [**Report a Bug**](https://github.com/mr-kelly/mato/issues) • [**Follow Roadmap**](docs/todos/TODO.md)

**Made with 🏖️ for developers who value clarity.**

</div>
