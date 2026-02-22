<div align="center">

<img src="logo.svg" alt="Mato Logo" width="160">

# Mato

### 🏖️ The Multi-Agent Terminal Office
**Elevate your terminal workflow with real-time activity intelligence.**

[![Rust](https://img.shields.io/badge/rust-2021-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue?style=for-the-badge)](LICENSE)
[![Version](https://img.shields.io/badge/version-v0.2.0-64A0FF?style=for-the-badge)](https://github.com/YOUR_USERNAME/mato/releases)
[![Stars](https://img.shields.io/github/stars/YOUR_USERNAME/mato?style=for-the-badge&color=gold)](https://github.com/YOUR_USERNAME/mato/stargazers)

[**Why Mato?**](#-the-vision) • [**Showcase**](#-showcase) • [**Quick Start**](#-get-started-in-60s) • [**Documentation**](#-pro-resources)

---

**Mato** is a high-performance terminal workspace designed for the era of AI agents. It transforms your CLI into a powerful "Office" where you can monitor parallel tasks, manage complex agent swarms, and maintain persistent sessions—all with zero keyboard conflicts and a beautiful, modern TUI.

</div>

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
- **Power User**: 250+ tabs for complex swarms.
- **Full-Stack**: Frontend, Backend, and DB in one view.
- **Solo Dev**: Focused, minimalist productivity.

</td>
</tr>
</table>

---

## 📽️ Showcase

> [!TIP]
> **Experience the Flow**: Watch how Mato's activity indicators eliminate the need for constant tab-switching during a complex multi-agent development session.

<div align="center">
  <img src="docs/demo.gif" alt="Mato Showcase" width="900" style="border-radius: 12px; border: 1px solid #1C1C2A;">
  <p align="center"><i>Real-time activity indicators syncing across desks and tabs.</i></p>
</div>

---

## 🚀 Get Started in 60s

### 1. Installation
```bash
# Clone and build with high-performance optimizations
git clone https://github.com/YOUR_USERNAME/mato
cd mato
cargo build --release

# Install to your path
sudo mv target/release/mato /usr/local/bin/
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

## ⌨️ Shortcut Philosophy

Mato follows the **"Rule of One"**: Only one key (`Esc`) is reserved by the system. Everything else belongs to your shell.

| Key | Action | Context |
| :--- | :--- | :--- |
| **`Esc`** | **Jump / Teleport** | Global |
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

[**Star this project**](https://github.com/YOUR_USERNAME/mato) • [**Report a Bug**](https://github.com/YOUR_USERNAME/mato/issues) • [**Follow Roadmap**](docs/todos/TODO.md)

**Made with 🏖️ for developers who value clarity.**

</div>
