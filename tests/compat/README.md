# Compatibility Test Suite

## Quick Smoke Test

```bash
bash tests/compat/smoke_test.sh
```

Runs automated checks for:
- Shell basics (color, unicode, large output)
- TUI apps (vim, nvim, htop, less, man)
- Alt-screen transitions
- Input sequences

## Manual Tests

These require visual verification inside a running mato session:

### Resize Test
1. Open vim: `vim`
2. Resize the terminal window
3. Verify vim redraws correctly (with `resize_strategy = "sync"`)
4. Exit: `:q!`

### lazygit
1. Run `lazygit` in a git repo
2. Navigate with arrow keys
3. Verify UI renders correctly
4. Exit: `q`

### SSH nested
1. SSH to a remote host
2. Verify shell works normally inside mato
3. Run a TUI app (htop/vim) over SSH

### Mouse
1. Open htop
2. Click on processes
3. Verify mouse events are forwarded correctly
