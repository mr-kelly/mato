# Mato Keyboard Shortcuts

> **Philosophy**: Minimal shortcuts, maximum productivity. Everything starts with `Esc`.

## 🎯 The One Key Rule

**In terminal focus, only `Esc` is special. Everything else goes to your shell.**

This means:
- ✅ `Ctrl+A/E/K/U` — Your bash shortcuts work
- ✅ `Ctrl+R` — Reverse search works
- ✅ `Ctrl+C/D/Z` — Process control works
- ✅ All your muscle memory preserved

**No prefix key. No mode confusion. Just `Esc` when you need to navigate.**

## Essential Shortcuts (You Only Need These)

| Key | Action | When |
|-----|--------|------|
| `Esc` | **Jump Mode** | In terminal → Navigate anywhere |
| `a-z` | Jump to task/tab | In Jump Mode → Instant jump |
| `←` or `↑` | Switch focus | In Jump Mode → Traditional navigation |
| `q` | Quit | In sidebar → Exit Mato |

**That's it.** Everything else is optional convenience.

## Optional Shortcuts

### Global (Work Anywhere)

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Ctrl+Z` | Suspend | Suspend Mato (resume with `fg`) |

## Sidebar Mode (Task List)

When focus is on the sidebar (left panel):

| Shortcut | Action | Description |
|----------|--------|-------------|
| `↑` / `↓` | Navigate | Move up/down in task list |
| `n` | New Task | Create a new task |
| `x` | Close Task | Delete current task |
| `r` | Rename Task | Rename current task |
| `Enter` | Focus Terminal | Switch to terminal content |
| `q` | Quit | Exit Mato |

## Topbar Mode (Tab List)

When focus is on the topbar (tab bar):

| Shortcut | Action | Description |
|----------|--------|-------------|
| `←` / `→` | Navigate | Move left/right in tab list |
| `n` | New Tab | Create a new tab |
| `x` | Close Tab | Delete current tab |
| `r` | Rename Tab | Rename current tab |
| `Enter` | Focus Terminal | Switch to terminal content |
| `Esc` | Back to Sidebar | Return to sidebar |

**Note**: `n` and `x` are **context-aware**:
- In **Sidebar** → New/Close **Task**
- In **Topbar** → New/Close **Tab**

Same keys, different context. Simple and consistent.

## Terminal Mode (Content)

When focus is on the terminal content:

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Esc` | **Jump Mode** | Enter Jump Mode for quick navigation |
| All other keys | Forward to Shell | Sent to terminal |

**Note**: In terminal mode, all keys except `Esc` are forwarded to the shell.

## Jump Mode 🎯

**Activated by**: Press `Esc` when in terminal focus

Jump Mode provides **dual navigation**:
1. **EasyMotion-style jumps** - Press a letter to instantly jump to any task/tab
2. **Arrow key focus switching** - Use `←` or `↑` to switch focus areas (backward compatible)

| Shortcut | Action | Description |
|----------|--------|-------------|
| `a-z` | Jump | Jump to labeled task/tab |
| `←` or `a` | Focus Sidebar | Switch to sidebar |
| `↑` or `w` | Focus Topbar | Switch to topbar |
| `Esc` | Cancel | Exit Jump Mode |

### How Jump Mode Works

1. **Activate**: Press `Esc` while in terminal focus
2. **Labels Appear**: All visible tasks and tabs show letter labels `[a]`, `[b]`, `[c]`, etc.
3. **Choose Action**:
   - Press a **letter** → Jump to that task/tab
   - Press **arrow key** → Switch focus area
   - Press **Esc** → Cancel
4. **Done**: Jump Mode exits automatically after any action

### Example

```
Sidebar:              Topbar:
[a] Development       [d] Terminal 1  [e] Terminal 2  [f] Server
[b] Testing           
[c] Documentation     
```

**Letter jumps**:
- Press `b` → Switch to "Testing" task
- Press `e` → Switch to "Terminal 2" tab

**Arrow keys** (backward compatible):
- Press `←` → Focus sidebar
- Press `↑` → Focus topbar

**Benefits**:
- ⚡ **Instant navigation** - single keypress to any location
- 🔄 **Backward compatible** - arrow keys work like before
- 👀 **Visual feedback** - see all available targets
- 🎯 **Flexible** - choose between jump or focus switch

## Rename Mode

When renaming a task or tab:

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Enter` | Confirm | Save the new name |
| `Esc` | Cancel | Discard changes |
| `Backspace` | Delete | Delete last character |
| Any character | Type | Add character to name |

## Mouse Support

| Action | Effect |
|--------|--------|
| Click task | Select task |
| Double-click task | Select task + focus terminal |
| Click tab | Switch to tab |
| Double-click tab | Rename tab |
| Click `＋` (topbar) | New tab |
| Click `＋` (sidebar) | New task |
| Scroll in sidebar | Navigate tasks |

## Focus Modes

Mato has three focus modes:

1. **Sidebar** (default) - Task list on the left
2. **Topbar** - Tab bar at the top
3. **Content** - Terminal content

### Focus Navigation

```
Sidebar ←→ Topbar
   ↓         ↓
   Content ←┘
```

- From **Sidebar**: `Enter` → Content
- From **Topbar**: `Enter` → Content, `Esc` → Sidebar
- From **Content**: `Esc` → Switch mode → `←/a` (Sidebar) or `↑/w` (Topbar)

## Quick Reference Card

### Most Used Shortcuts

```
Navigation:
  Alt+1-9         Quick tab switch
  Ctrl+PageUp/Dn  Previous/Next tab
  ↑↓←→            Navigate UI

Task Management:
  n               New task
  t               New tab
  x/w             Close task/tab
  r               Rename

Focus:
  Enter           Focus terminal
  Esc             Switch mode
  Esc → ←/a       Sidebar
  Esc → ↑/w       Topbar

Exit:
  q               Quit (from sidebar)
  Ctrl+Z          Suspend
```

## Tips & Tricks

### 1. Quick Tab Switching
Use `Alt+1-9` to instantly jump to any of the first 9 tabs without leaving terminal mode.

### 2. Efficient Navigation
- Use `Ctrl+PageUp/PageDown` for sequential tab navigation
- Use `Alt+Number` for direct tab access

### 3. Keyboard-Only Workflow
You can use Mato entirely without a mouse:
1. Start in sidebar (default)
2. Navigate tasks with `↑↓`
3. Press `Enter` to focus terminal
4. Use `Esc` → `↑` to access tabs
5. Use `←→` to switch tabs
6. Press `Enter` to return to terminal

### 4. Mouse + Keyboard Hybrid
- Use mouse for quick task/tab selection
- Use keyboard for terminal work and shortcuts

### 5. Renaming
- Double-click any task/tab name to rename
- Or use `r` key when focused on sidebar/topbar

## Customization

Currently, keybindings are fixed. Custom keybindings will be added in a future version.

## Comparison with Other Tools

| Action | Standard Multiplexers | Mato |
|--------|------|------|
| Prefix | `Ctrl+B` | None (direct keys) |
| New window | `Ctrl+B c` | `n` (sidebar) or `t` (topbar) |
| Next window | `Ctrl+B n` | `→` or `Ctrl+PageDown` |
| Previous window | `Ctrl+B p` | `←` or `Ctrl+PageUp` |
| Select window | `Ctrl+B 0-9` | `Alt+1-9` |
| Rename | `Ctrl+B ,` | `r` |
| Detach | `Ctrl+B d` | `q` (quit, daemon keeps running) |

**Key Difference**: Mato doesn't use a prefix key, making shortcuts more direct and faster to use.

## Troubleshooting

### Shortcuts Not Working?

1. **Check focus mode**: Some shortcuts only work in specific modes
2. **Terminal mode**: Remember to press `Esc` first to enter switch mode
3. **Rename mode**: Press `Esc` to exit rename mode if stuck

### Alt Key Not Working?

Some terminals may intercept `Alt` key combinations. Try:
- Use `Ctrl+PageUp/PageDown` instead
- Check your terminal emulator settings
- Ensure "Use Option as Meta key" is enabled (macOS Terminal.app)

### Mouse Not Working?

- Mouse support should work in most modern terminals
- If not working, use keyboard shortcuts instead
- Check if your terminal supports mouse events

## See Also

- [README.md](../README.md) - General documentation
- [Configuration Guide](CONFIGURATION.md) - Config file options
- [Troubleshooting Guide](TROUBLESHOOTING.md) - Common issues
