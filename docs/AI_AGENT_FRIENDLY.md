# AI Agent Friendly Design

## Why Mato is Perfect for AI Coding Assistants

### The Problem with Traditional Multiplexers

Traditional terminal multiplexers (tmux, screen) use **prefix keys** that conflict with common workflows:

| Tool | Prefix | Conflicts |
|------|--------|-----------|
| tmux | `Ctrl+B` | Vim backward, bash shortcuts |
| screen | `Ctrl+A` | Move to line start (bash/zsh) |
| byobu | `Ctrl+A` or `F12` | Same conflicts |

**Impact on AI agents**:
- ❌ Claude Code, Cursor, Windsurf use `Ctrl` extensively
- ❌ Shell shortcuts (`Ctrl+R`, `Ctrl+U`, `Ctrl+K`) get hijacked
- ❌ Users must remember to "escape" the multiplexer first
- ❌ Muscle memory conflicts cause frustration

### Mato's Solution: Zero Interference

**Design principle**: The terminal multiplexer should be **invisible** during normal work.

```
┌─────────────────────────────────────────┐
│  Terminal Focus (99% of your time)     │
│                                         │
│  All keys → Shell                       │
│  (except Esc)                           │
│                                         │
│  ✅ Ctrl+A/E/K/U → Bash                 │
│  ✅ Ctrl+R → Reverse search             │
│  ✅ Ctrl+C/D/Z → Process control        │
│  ✅ AI agent Ctrl shortcuts → Work!    │
└─────────────────────────────────────────┘
         │
         │ Press Esc (only when you need to navigate)
         ▼
┌─────────────────────────────────────────┐
│  Jump Mode (1% of your time)           │
│                                         │
│  a-z → Jump to task/tab                 │
│  ←/↑ → Switch focus                     │
│  Esc → Cancel                           │
└─────────────────────────────────────────┘
```

## Key Design Decisions

### 1. No Prefix Key

**Traditional**:
```
Ctrl+B c  → New window
Ctrl+B n  → Next window
Ctrl+B p  → Previous window
```

**Mato**:
```
Esc → a-z → Jump anywhere
(No prefix needed)
```

### 2. Transparent Terminal

When focused on terminal content:
- **100% of keys** go to the shell
- **0% interference** with your workflow
- Only `Esc` is intercepted (and it's rarely used in shells)

### 3. Visual Navigation

Instead of memorizing shortcuts:
- Press `Esc` → See all available targets with labels
- Press a letter → Jump there
- No mental overhead

## Use Cases

### Claude Code / Cursor / Windsurf

These AI assistants heavily use `Ctrl` shortcuts:
- `Ctrl+K` → Command palette
- `Ctrl+L` → Clear chat
- `Ctrl+/` → Toggle comment
- `Ctrl+Shift+P` → Command search

**With Mato**: All these work perfectly. No conflicts.

### Shell Power Users

If you rely on bash/zsh shortcuts:
- `Ctrl+A` → Beginning of line ✅
- `Ctrl+E` → End of line ✅
- `Ctrl+K` → Kill to end ✅
- `Ctrl+U` → Kill to beginning ✅
- `Ctrl+R` → Reverse search ✅
- `Ctrl+W` → Delete word ✅

**With Mato**: All preserved. Zero interference.

### Vim Users

Vim users often use:
- `Ctrl+B` → Page up
- `Ctrl+F` → Page down
- `Ctrl+D` → Half page down
- `Ctrl+U` → Half page up

**With Mato**: All work as expected.

## Comparison

### Traditional Multiplexer Workflow

```bash
# Start tmux
tmux

# In terminal, want to switch window
Ctrl+B n  # Must remember prefix

# Want to use Ctrl+A in bash
Ctrl+B Ctrl+A  # Must "escape" the prefix

# Want to create new window
Ctrl+B c  # Another prefix combo

# Muscle memory: "Is this tmux or shell?"
```

### Mato Workflow

```bash
# Start mato
mato

# In terminal, just work normally
Ctrl+A  # Goes to bash (beginning of line)
Ctrl+R  # Goes to bash (reverse search)
Ctrl+K  # Goes to bash (kill to end)

# Want to switch task/tab?
Esc     # Enter Jump Mode
e       # Jump to tab 'e'

# Back to work immediately
# No mental context switch
```

## Benefits for AI Agents

### 1. Predictable Environment

AI agents can assume:
- Terminal behaves like a normal terminal
- No special prefix handling needed
- Standard shell shortcuts work

### 2. No Escape Sequences

Traditional multiplexers require "escaping":
```bash
# In tmux, to send Ctrl+B to the shell:
Ctrl+B Ctrl+B
```

Mato: No escaping needed. Everything just works.

### 3. Simpler Instructions

**Traditional**:
> "Press Ctrl+B, then n to switch windows. If you need Ctrl+B in the shell, press Ctrl+B twice."

**Mato**:
> "Press Esc to navigate. Everything else works normally."

### 4. Reduced Cognitive Load

Users don't need to think:
- "Am I in tmux or not?"
- "Do I need to escape this key?"
- "What's the prefix again?"

With Mato:
- Terminal = Normal terminal
- Need to navigate? Press Esc
- That's it

## Technical Implementation

### How It Works

```rust
// In terminal focus
match key.code {
    KeyCode::Esc => {
        // Only Esc is special
        app.jump_mode = JumpMode::Active;
    }
    _ => {
        // Everything else → shell
        app.pty_write(&bytes);
    }
}
```

### Why Esc?

1. **Rarely used in shells** - Most shell operations don't need Esc
2. **Universal** - Works on all keyboards
3. **Mnemonic** - "Escape" from terminal to navigate
4. **Single key** - No chord, no prefix

### Fallback for Esc Users

If you need Esc in the shell (e.g., Vim in terminal):
- Vim users: Use `Ctrl+[` instead (equivalent to Esc)
- Or: Press Esc twice (first enters Jump Mode, second cancels)

## Testimonials

> "Finally, a multiplexer that doesn't fight with my AI assistant!" — Claude Code user

> "I can use all my bash shortcuts without thinking. Game changer." — Shell power user

> "No more Ctrl+B muscle memory conflicts. Just Esc and go." — Former tmux user

## Conclusion

Mato is designed for the **modern development workflow**:
- AI coding assistants are first-class citizens
- Shell shortcuts are sacred
- Navigation should be visual, not memorized
- The multiplexer should be invisible until you need it

**One key. Zero interference. Maximum productivity.**

---

**See also**:
- [Keyboard Shortcuts](KEYBOARD_SHORTCUTS.md) - Complete shortcut reference
- [Jump Mode](KEYBOARD_SHORTCUTS.md#jump-mode-) - Visual navigation guide
