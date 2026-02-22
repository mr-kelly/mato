# Mato Terminal Runtime Roadmap & Capability Audit

**Date**: 2026-02-22  
**Version Baseline**: v0.7.1 (post-sync code baseline)  
**Purpose**: Full capability checklist, current-state audit, and prioritized gap roadmap for coding agents

## Legend

- `[x]` Implemented and usable
- `[ ]` Missing
- `[~]` Partially implemented (usable but incomplete)

---

## 0. Product Goals & Invariants

### Goal

- A modern terminal runtime + TUI with multi-session, multi-task, and multi-PTY support.

### Current Invariants

- `[x]` Best-effort terminal restoration on exit/crash (`TerminalGuard` + panic/signal cleanup)
- `[x]` Client disconnect does not stop daemon-side PTYs (basic attach/detach behavior)
- `[~]` Session continuity: client reconnect works, but PTY process state is lost after daemon restart
- `[~]` CPU control: adaptive polling exists (80ms/200ms), but no formal performance SLA yet

---

## 1. Terminal & PTY Fundamentals

### 1.1 PTY and Child Process Management

- `[x]` PTY spawn (`portable_pty`)
- `[x]` PTY read/write loop
- `[x]` Child-exit detection (`try_wait`) with auto-respawn
- `[x]` Shell resolution strategy (`$SHELL` -> passwd shell -> `/bin/sh`)
- `[x]` `TERM=xterm-256color`
- `[ ]` Per-tab `cwd/env/argv/shell` configuration
- `[~]` Zombie handling: `try_wait` exists, no centralized `SIGCHLD` reaper model

### 1.2 Terminal State & Recovery

- `[x]` Raw/cooked mode switching
- `[x]` Restore cursor/mouse/bracketed-paste/alt-screen
- `[x]` Panic/signal best-effort cleanup
- `[~]` Title restore policy is limited

---

## 2. Input System

### 2.1 Keyboard Input Parsing

- `[x]` Core key/modifier/function handling via crossterm
- `[x]` Content-area key encoding (arrows/F-keys/Home/End/PageUp/PageDown/Backspace)
- `[~]` UTF-8/IME behavior depends on crossterm; no dedicated IME matrix yet
- `[ ]` Custom Alt-vs-Esc timeout disambiguation (byte-stream level)
- `[ ]` Repeat/long-press throttling policy

### 2.2 Bracketed Paste

- `[x]` Bracketed paste mode detection + wrapped paste send
- `[x]` `Event::Paste` path in UI loop
- `[~]` No standalone global “paste-mode shortcut bypass” policy model

### 2.3 Mouse

- `[x]` Click/drag/scroll handling
- `[x]` SGR mouse passthrough to PTY in content area
- `[~]` Cross-terminal compatibility matrix not systematized

### 2.4 Focus Events

- `[x]` Focus in/out sequences (`\x1b[I` / `\x1b[O`)
- `[ ]` Focus-aware refresh throttling policy

---

## 3. Output & Rendering

### 3.1 ANSI/VT Parsing & Screen Model

- `[x]` ANSI/VT parsing (alacritty + vt100 emulators)
- `[x]` 16/256/truecolor
- `[x]` Attributes: bold/italic/underline/inverse/strikethrough/hidden/dim
- `[x]` Wide-char and combining-char handling (`display_width`, `zerowidth`)
- `[~]` OSC coverage beyond title/reset/bell is not fully mapped
- `[ ]` OSC 8 hyperlink support in UI

### 3.2 Alternate Screen

- `[x]` Enter/leave alt-screen and restore behavior
- `[x]` Baseline compatibility with full-screen TUIs (vim/lazygit class)

### 3.3 Resize

- `[x]` `Event::Resize` capture
- `[x]` Client layout updates
- `[~]` PTY winsize sync is intentionally disabled in daemon (content-preservation tradeoff)
- `[x]` Resize debouncing/pending-apply strategy

### 3.4 Rendering Strategy

- `[x]` Frame throttling (active/idle)
- `[~]` Relies on ratatui diffing, but no explicit Mato rendering SLA
- `[ ]` Explicit backpressure policy for high-throughput output

---

## 4. Multiplexer Core

### 4.1 Session Model

- `[x]` Office / Desk / Tab hierarchy
- `[x]` One PTY + screen buffer per tab
- `[ ]` Split panes

### 4.2 Attach/Detach

- `[x]` Daemon + client attach model
- `[x]` Work continues when client disconnects
- `[~]` Multi-client attach works technically; collaboration semantics undefined
- `[ ]` Explicit session attach/detach CLI UX

### 4.3 Input Routing & Key Conflicts

- `[x]` Focused routing model (Sidebar/Topbar/Content)
- `[x]` Jump mode
- `[~]` No tmux-style prefix mode (intentional minimalist model)
- `[ ]` Per-tab keymap/profile

---

## 5. Task & Agent Management (Differentiation)

- `[~]` Tab/desk-based task execution primitives exist (spawn/close)
- `[ ]` Formal task schema (`name/cmd/cwd/env/tags/restart_policy`)
- `[ ]` Task state machine (`starting/running/exited/failed`)
- `[ ]` Restart/backoff policy
- `[ ]` Concurrency limits
- `[ ]` Resource limits (e.g., cgroup)
- `[~]` Bell and activity indicators exist (spinner + bell)
- `[ ]` Desktop/webhook/Discord notifications

---

## 6. Plugin System

- `[ ]` Plugin API (commands/keybindings/widgets/hooks)
- `[ ]` Sandbox permission model
- `[ ]` Runtime choice (Lua/JS/WASM)
- `[ ]` Plugin compatibility/versioning policy

---

## 7. Config, Persistence, Portability

### 7.1 Configuration

- `[x]` TOML config foundation
- `[x]` Theme config
- `[~]` Partial reload path (daemon SIGHUP); limited client-side hot reload
- `[ ]` Configurable keymap

### 7.2 Persistence

- `[x]` Layout/metadata persistence (`state.json`)
- `[x]` Reconnect restores visible content while daemon is alive
- `[ ]` Daemon-restart screen-state restoration
- `[~]` Crash recovery is strong for terminal modes, limited for runtime state

---

## 8. Compatibility Risk Checklist (Current)

- `[x]` Baseline TERM handling
- `[~]` Backspace behavior variance not fully normalized
- `[~]` Alt-key variance not fully validated
- `[~]` CJK/emoji width still needs broad compatibility coverage
- `[~]` Mouse protocol compatibility matrix incomplete
- `[ ]` mosh-specific strategy
- `[~]` SSH-disconnect behavior benefits from daemon model, but lacks automated regression suite

---

## 9. Security & Robustness

- `[x]` Basic socket permissions (`0700`)
- `[x]` Daemon lock/pid/socket lifecycle management
- `[~]` Command-injection surface remains to formalize for future task schema
- `[ ]` Resource-leak detection framework (fd/pty/thread)
- `[ ]` Log redaction policy
- `[~]` Crash guard exists; no standardized diagnostics bundle/state dump

---

## 10. Tests & Acceptance

### 10.1 Compatibility Acceptance Matrix

- `[ ]` vim / nvim
- `[ ]` htop / btop
- `[ ]` less / man
- `[ ]` k9s
- `[ ]` lazygit
- `[ ]` ssh-inside / mosh-inside
- `[ ]` high-output stress (`yes | head -n 20000`)

### 10.2 Functional Acceptance

- `[x]` Work continues after detach/disconnect
- `[x]` Reconnect restores content while daemon remains alive
- `[~]` Resize behavior is stable with fixed-PTY strategy, but tradeoff remains
- `[x]` Bracketed paste baseline works
- `[x]` Terminal state recovery after crash works

---

## 11. Recommended Execution Order (for Coding Agents)

### P0: Shell/Terminal Core Experience (Top Priority)

1. Automated compatibility matrix (vim/nvim/htop/lazygit/less/high-output)
2. Input hardening (Alt/Esc boundaries, IME behavior, Backspace normalization)
3. Configurable resize strategy:
   - `safe_fixed_pty_size` (current)
   - `sync_pty_winsize` (experimental mode)
4. Backpressure + render SLA under high output

### P1: Multiplexer & Session Completion

1. Named sessions + explicit attach/detach CLI
2. Multi-client collaboration semantics (read-only/input ownership)
3. Configurable keymap while keeping AI-friendly defaults

### P2: Task/Agent Capability Framework

1. Task schema (`cmd/cwd/env/restart/backoff`)
2. Task state machine + events
3. Notification pipeline (bell -> desktop/webhook)

### P3: Plugin System

1. Minimal plugin API (commands/hooks)
2. Security boundaries
3. Runtime selection (WASM or Lua first)

---

## Ready-to-Assign Task Cards

1. `compat-matrix`: add `tests/compat/` with TUI smoke tests and capture scripts.
2. `input-hardening`: isolate input policy layer + Alt/Esc/Backspace/IME tests/docs.
3. `resize-mode`: add `resize_strategy` config and implement fixed/sync modes.
4. `session-cli`: design `mato attach/detach/sessions` commands.
5. `task-schema`: introduce task model + restart policy + state machine.
