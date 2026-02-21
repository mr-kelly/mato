# Terminal Persistence Tests

This directory contains tests to ensure terminal content persistence works correctly.

## Test Types

### Unit Tests (`terminal_persistence_tests.rs`)

Tests the `PtyProvider` directly without requiring a daemon:

- ✅ `test_terminal_content_persistence` - Content survives get_screen calls
- ✅ `test_resize_preserves_content_when_size_unchanged` - No-op resize doesn't clear screen
- ✅ `test_multiple_writes_accumulate` - Multiple writes are preserved
- ✅ `test_pty_survives_multiple_get_screen_calls` - Content persists across multiple reads
- ✅ `test_spawn_is_idempotent` - Second spawn doesn't affect existing PTY

**Run with:**
```bash
cargo test --test terminal_persistence_tests
```

### Integration Tests (`daemon_persistence_tests.rs`)

Tests the full client-daemon flow (requires running daemon):

- `test_daemon_terminal_persistence` - Content survives client reconnection
- `test_daemon_multiple_tabs` - Multiple tabs maintain separate content
- `test_resize_preserves_content` - Content survives window resize (critical!)

**Run with:**
```bash
# Start daemon first
cargo run -- --daemon --foreground

# In another terminal
cargo test --test daemon_persistence_tests -- --ignored
```

### Automated Test Runner

Run all tests automatically:

```bash
./tests/run_persistence_tests.sh
```

This script will:
1. Run unit tests
2. Start daemon if needed
3. Run integration tests
4. Clean up

## What These Tests Verify

### ✅ Content Persistence
- Terminal content is preserved when calling `get_screen()` multiple times
- Content survives client disconnection and reconnection
- Multiple tabs maintain separate content

### ✅ Resize Behavior
- Resizing to the same size is a no-op (doesn't clear screen)
- Only actual size changes trigger resize

### ✅ PTY Lifecycle
- `spawn()` is idempotent (calling twice doesn't create new PTY)
- PTY continues running in daemon after client disconnects
- Multiple clients can connect to the same PTY
- **PTY size is fixed** - resize messages are ignored to preserve content

## Known Limitations

These tests verify current behavior, but there are known limitations:

1. **Resize to different size may clear screen** - This is a limitation of the vt100 parser
2. **No scrollback buffer** - Only current screen is tested
3. **Daemon restart loses state** - Tests assume daemon stays running

See `docs/TERMINAL_PERSISTENCE.md` for details.

## CI Integration

Add to `.github/workflows/test.yml`:

```yaml
- name: Run persistence tests
  run: |
    cargo test --test terminal_persistence_tests
    # Integration tests require daemon, skip in CI for now
```

## Debugging Failed Tests

If tests fail:

1. **Check daemon logs**: `tail -f ~/.local/state/mato/daemon.log`
2. **Verify PTY creation**: Look for "Spawning new tab" in logs
3. **Check timing**: Increase sleep durations if tests are flaky
4. **Run with output**: `cargo test -- --nocapture`

## Adding New Tests

When adding features that affect terminal persistence:

1. Add unit test in `terminal_persistence_tests.rs`
2. Add integration test in `daemon_persistence_tests.rs` if needed
3. Update this README
4. Run `./tests/run_persistence_tests.sh` to verify

---

**Last Updated**: 2026-02-21  
**Test Coverage**: 5 unit tests, 2 integration tests
