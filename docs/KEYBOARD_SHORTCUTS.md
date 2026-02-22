# Mato Keyboard Shortcuts

> Philosophy: minimal shortcuts, predictable focus, visual jump labels.

## Focus Modes

Mato has three focus areas:

1. **Sidebar** - desk list
2. **Topbar** - tab list
3. **Content** - terminal content

## Essential Shortcuts

| Key | Action | Scope |
|-----|--------|-------|
| `Esc` | Enter Jump Mode | Any focus |
| `a-z/A-Z` | Jump to labeled target | Jump Mode |
| `←` | Focus Sidebar | Jump Mode |
| `↑` | Focus Tabbar | Jump Mode |
| `q` | Quit Mato | Sidebar / Topbar / Jump Mode |

## Sidebar Mode (Desk List)

| Key | Action |
|-----|--------|
| `↑` / `↓` | Select desk |
| `n` | New desk |
| `x` | Close desk |
| `r` | Rename selected desk |
| `o` | Open office selector |
| `s` | Open settings |
| `Enter` | Focus Content |
| `Esc` | Enter Jump Mode |
| `q` | Quit |

## Topbar Mode (Tab List)

| Key | Action |
|-----|--------|
| `←` / `→` | Select tab |
| `n` | New tab |
| `x` | Close selected tab |
| `r` | Rename selected tab |
| `Enter` | Focus Content |
| `Esc` | Enter Jump Mode |
| `q` | Quit |

## Content Mode (Terminal)

| Key | Action |
|-----|--------|
| `Esc` | Enter Jump Mode |
| `Shift+PageUp` | Scrollback up |
| `Shift+PageDown` | Scrollback down |
| Other keys | Forward to shell |

## Jump Mode

### What appears

- Jump labels are `a-z` + `A-Z` (up to 52 targets).
- Labels cover desks and visible tabs.

### Target allocation strategy

- **From Content**: balanced interleaving between tabs and desks.
- **From Topbar**: tabs first, then desks.
- **From Sidebar**: desks first, then tabs.

### Selection result

- Select a **desk label** -> switch desk, focus becomes **Sidebar**.
- Select a **tab label** -> switch tab, focus becomes **Topbar**.

### Keys in Jump Mode

| Key | Action |
|-----|--------|
| `a-z/A-Z` | Jump to target |
| `←` | Focus Sidebar (without selecting a label) |
| `↑` | Focus Tabbar (without selecting a label) |
| `Esc` | Cancel Jump Mode |
| `q` | Quit Mato |

## Global

| Key | Action |
|-----|--------|
| `Ctrl+Z` | Suspend Mato (resume with `fg`) |
| `Alt+1-9` | Quick switch to tab 1-9 |

## Mouse

| Action | Effect |
|--------|--------|
| Click desk | Select desk |
| Double-click desk | Select desk + focus Content |
| Click tab | Switch tab |
| Double-click tab | Rename tab |
| Click `＋` in topbar | New tab |
| Click office area | Open office selector |
| Scroll in sidebar | Move desk selection |

## Office Operations

| Goal | Keys |
|------|------|
| Switch office | `o` -> select -> `Enter` |
| Rename office | `o` -> select -> `r` |
| Create office | `o` -> select `＋ New Office` -> `Enter` |
| Delete office | `o` -> select -> `d` -> type name -> `Enter` |

## Notes

- `w/a` are **not** focus shortcuts in Jump Mode.
- Focus switching in Jump Mode is arrow-only (`←`, `↑`) to avoid conflicts with letter labels.
- If a tab is not visible in the topbar viewport, it will not get a jump label until visible.

## See Also

- [README.md](../README.md)
- [docs/AI_AGENT_FRIENDLY.md](AI_AGENT_FRIENDLY.md)
- [docs/TERMINAL_PERSISTENCE.md](TERMINAL_PERSISTENCE.md)
