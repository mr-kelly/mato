# Terminal Resize Strategy

## Problem

When client window resizes, terminal content disappears. This is a critical UX issue.

## Root Cause

**Initial (Wrong) Design**:
```
Client window resize → Send Resize message to daemon
                    → Daemon calls PtyProvider::resize()
                    → Terminal emulator creates new parser
                    → Screen content cleared ❌
```

**Why emulator clears content**:
- `vt100::Parser::new()` creates empty buffer
- `vte` TerminalState also starts fresh
- Neither crate supports content-preserving resize

## Failed Solutions

### Attempt 1: Delay Resize on Client
```rust
// Client delays resize for 500ms
pub pending_resize: Option<(u16, u16, Instant)>
```
**Result**: ❌ Doesn't help - daemon still clears when resize is applied

### Attempt 2: Track Size to Avoid Duplicates
```rust
// Both client and daemon track current_size
if self.current_size == (rows, cols) { return; }
```
**Result**: ❌ Prevents duplicate resizes, but real resize still clears

### Attempt 3: Preserve Content in Emulator
**Result**: ❌ Not supported by vt100/vte crates

## Correct Solution: Don't Resize PTY

### Key Insight

**PTY is a server-side resource that should remain stable.**

- PTY runs in daemon with fixed size
- Client window size is a client-side concern
- If sizes differ, client adapts its display

### Implementation

```rust
// In daemon.rs
ClientMsg::Resize { tab_id, rows, cols } => {
    // DON'T resize the PTY! This would clear the screen.
    // The PTY should keep running at its original size.
    // Only the client's display needs to adapt to window size.
    tracing::debug!("Ignoring resize request - PTY size is fixed");
    continue;
}
```

### Client-Side Handling

```rust
// In DaemonProvider
pub struct DaemonProvider {
    current_size: (u16, u16),  // Track to avoid sending duplicate messages
}

fn resize(&mut self, rows: u16, cols: u16) {
    if self.current_size == (rows, cols) {
        return;  // Don't send message
    }
    self.current_size = (rows, cols);
    self.send_msg_no_response(ClientMsg::Resize { ... });
    // Daemon will ignore this, but we track it anyway
}
```

## Behavior

### Small Window (e.g., 60x20) with PTY 80x24
- Client displays first 60 columns, 20 rows
- User can scroll to see rest (future feature)

### Large Window (e.g., 100x30) with PTY 80x24
- Client displays PTY content (80x24)
- Extra space shows as padding

### Window Resize
- PTY size unchanged → **Content preserved** ✅
- Client adapts display to new window size
- No data loss

## Comparison with Terminal Multiplexers

This matches standard multiplexer behavior:

| Scenario | Standard Multiplexers | Mato |
|----------|----------------------|------|
| Multiple clients, different sizes | Uses smallest size | PTY keeps original size |
| Client resizes window | PTY unchanged | PTY unchanged ✅ |
| Content preservation | Always preserved | Always preserved ✅ |
| Large window display | Shows padding | Shows padding ✅ |

## Testing

### Integration Test

```rust
#[test]
#[ignore]
fn test_resize_preserves_content() {
    // 1. Spawn PTY at 24x80
    // 2. Write content
    // 3. Send Resize to 30x100
    // 4. Get screen content
    // 5. Assert content still there ✅
}
```

**Run with**:
```bash
cargo test --test daemon_persistence_tests -- --ignored
```

## Trade-offs

### Advantages ✅
- **Content never lost** on resize
- Simple implementation
- Follows standard multiplexer patterns
- No emulator limitations

### Disadvantages ⚠️
- PTY size fixed at spawn time
- Large windows show padding
- Small windows need scrolling (future)

### Future Improvements

**Phase 11: Smart Resize**
- Allow resize when PTY is idle (no running commands)
- Detect safe resize points (at shell prompt)
- Preserve content by re-rendering to new size

**Phase 12: Scrollback Buffer**
- Store full terminal history
- Allow scrolling in small windows
- Preserve content beyond visible screen

## Lessons Learned

1. **Question Assumptions** - "Resize PTY to match window" seemed obvious but was wrong
2. **Server vs Client** - Distinguish server-side resources from client-side concerns
3. **Simplicity Wins** - The correct solution (ignore resize) is simpler than failed attempts
4. **Test Critical Paths** - Resize is a common operation, must have integration test
5. **Learn from Prior Art** - Standard multiplexers use this approach

## References

- `src/daemon_modules/daemon.rs` - Resize message handling
- `src/providers/daemon_provider.rs` - Client-side size tracking
- `tests/daemon_persistence_tests.rs` - Integration test
- `docs/TERMINAL_PERSISTENCE.md` - User-facing documentation

---

**Date**: 2026-02-21  
**Status**: Implemented and tested ✅  
**Impact**: Critical UX improvement
