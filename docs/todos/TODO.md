# Mato TODO (Execution Backlog)

**Last Updated**: 2026-02-24  
**Version Baseline**: v0.9.5  
**Source of Truth**: `docs/todos/roadmap.md`

This file is the execution-oriented backlog synced from `roadmap.md`.

## Legend

- `[ ]` Not started
- `[~]` In progress / partial
- `[x]` Completed

---

## P0: Shell & Terminal Core Experience

### Compatibility Matrix
- [ ] Add automated compat smoke tests in `tests/compat/` for: `vim/nvim`, `htop/btop`, `less/man`, `k9s`, `lazygit`, `ssh-inside`, high-output stress.
- [ ] Add repeatable capture scripts and pass/fail criteria.

### Input Hardening
- [ ] Define Alt/Esc disambiguation policy and tests.
- [ ] Document and validate IME behavior boundaries.
- [ ] Normalize Backspace behavior across terminals.

### Resize Strategy (Fixed vs Sync)
- [~] Implementation exists (`fixed`/`sync`) with constraints.
- [ ] Document exact multi-client semantics and expected behavior.
- [ ] Add acceptance tests for `fixed` and `sync` modes.

### Throughput & Backpressure
- [~] Coalescing + push/diff dedup are implemented.
- [ ] Define explicit backpressure policy contract.
- [ ] Add stress acceptance target and regression checks.

---

## P1: Multiplexer & Session Completion

### Session UX
- [ ] Design and implement explicit CLI: `mato sessions`, `mato attach`, `mato detach`.
- [ ] Add named sessions lifecycle (create/list/switch/close).

### Multi-Client Collaboration Semantics
- [ ] Define input ownership policy (single-writer vs shared).
- [ ] Define read-only attach mode behavior.
- [ ] Add tests for multi-client contention cases.

### Keymap Configurability
- [ ] Add configurable keymap/profile support.
- [ ] Preserve AI-friendly defaults as baseline profile.

---

## P2: Graphics Protocol Hardening

### Current State
- [x] Graphics/APC passthrough pipeline exists.
- [x] Client emits pending graphics sequences after render.

### Remaining Work
- [ ] Expand compatibility matrix (kitty/ghostty/wezterm/iTerm2 and edge cases).
- [ ] Define explicit fallback/disable behavior on unsupported terminals.
- [ ] Add regression tests for mixed text + graphics flows.

---

## P3: Task/Agent Capability Framework

- [ ] Define formal task schema (`name/cmd/cwd/env/tags/restart_policy`).
- [ ] Implement task state machine (`starting/running/exited/failed`).
- [ ] Implement restart + backoff policy.
- [ ] Add notification pipeline (bell -> desktop/webhook/Discord).

---

## P4: Plugin System

- [ ] Minimal plugin API (commands/hooks/widgets).
- [ ] Sandbox permission model.
- [ ] Runtime choice and versioning policy (WASM/Lua/JS).

---

## Task Cards (Ready to Assign)

- [ ] `compat-matrix`: add `tests/compat/` smoke tests + capture tooling.
- [ ] `input-hardening`: isolate input policy layer + Alt/Esc/Backspace/IME tests/docs.
- [ ] `resize-mode-docs`: document current resize behavior + add fixed/sync acceptance tests.
- [ ] `session-cli`: design/implement attach-detach-session CLI.
- [ ] `task-schema`: introduce task model + lifecycle + restart policy.
- [ ] `graphics-compat`: extend graphics compatibility tests and fallback behavior.

---

## Notes

- Keep this file concise and execution-focused.
- For capability audit and rationale, update `docs/todos/roadmap.md`.
- For long-term state persistence design, see `docs/todos/FUTURE_STATE_PERSISTENCE.md`.
