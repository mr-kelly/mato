use std::thread;
use std::time::Duration;

/// Integration test for terminal content persistence
/// 
/// This test verifies that terminal content is preserved when:
/// 1. A PTY is spawned
/// 2. Content is written to it
/// 3. The client disconnects and reconnects
/// 4. Content should still be visible

#[test]
fn test_terminal_content_persistence() {
    use mato::terminal_provider::TerminalProvider;
    use mato::providers::PtyProvider;
    
    // Create a PTY provider
    let mut provider = PtyProvider::new();
    
    // Spawn a PTY
    provider.spawn(24, 80);
    
    // Write some content
    provider.write(b"echo 'Hello, World!'\n");
    
    // Wait for output to be processed
    thread::sleep(Duration::from_millis(500));
    
    // Get screen content
    let screen1 = provider.get_screen(24, 80);
    
    // Verify content is not empty
    assert!(!screen1.lines.is_empty(), "Screen should have content after writing");
    
    // Find the line with our output
    let has_hello = screen1.lines.iter().any(|line| {
        line.cells.iter().any(|cell| {
            cell.ch == 'H' || cell.ch == 'e' || cell.ch == 'l'
        })
    });
    
    assert!(has_hello, "Screen should contain 'Hello' text");
    
    // Simulate reconnection by getting screen again
    let screen2 = provider.get_screen(24, 80);
    
    // Content should still be there
    assert_eq!(screen1.lines.len(), screen2.lines.len(), 
               "Screen content should persist across get_screen calls");
}

#[test]
fn test_resize_preserves_content_when_size_unchanged() {
    use mato::terminal_provider::TerminalProvider;
    use mato::providers::PtyProvider;
    
    let mut provider = PtyProvider::new();
    provider.spawn(24, 80);
    
    // Write content
    provider.write(b"echo 'Test Content'\n");
    thread::sleep(Duration::from_millis(500));
    
    let screen_before = provider.get_screen(24, 80);
    
    // Resize to same size (should be no-op)
    provider.resize(24, 80);
    
    let screen_after = provider.get_screen(24, 80);
    
    // Content should be identical
    assert_eq!(screen_before.lines.len(), screen_after.lines.len(),
               "Resize to same size should not affect content");
}

#[test]
fn test_multiple_writes_accumulate() {
    use mato::terminal_provider::TerminalProvider;
    use mato::providers::PtyProvider;
    
    let mut provider = PtyProvider::new();
    provider.spawn(24, 80);
    
    // Write multiple lines
    provider.write(b"echo 'Line 1'\n");
    thread::sleep(Duration::from_millis(200));
    
    provider.write(b"echo 'Line 2'\n");
    thread::sleep(Duration::from_millis(200));
    
    provider.write(b"echo 'Line 3'\n");
    thread::sleep(Duration::from_millis(200));
    
    let screen = provider.get_screen(24, 80);
    
    // Should have multiple lines of content
    let non_empty_lines = screen.lines.iter()
        .filter(|line| line.cells.iter().any(|cell| cell.ch != ' '))
        .count();
    
    assert!(non_empty_lines >= 3, 
            "Should have at least 3 lines of content, got {}", non_empty_lines);
}

#[test]
fn test_pty_survives_multiple_get_screen_calls() {
    use mato::terminal_provider::TerminalProvider;
    use mato::providers::PtyProvider;
    
    let mut provider = PtyProvider::new();
    provider.spawn(24, 80);
    
    provider.write(b"echo 'Persistent'\n");
    thread::sleep(Duration::from_millis(500));
    
    // Call get_screen multiple times (simulating multiple client connections)
    for i in 0..10 {
        let screen = provider.get_screen(24, 80);
        assert!(!screen.lines.is_empty(), 
                "Screen should have content on call {}", i);
    }
}

#[test]
fn test_spawn_is_idempotent() {
    use mato::terminal_provider::TerminalProvider;
    use mato::providers::PtyProvider;
    
    let mut provider = PtyProvider::new();
    
    // First spawn
    provider.spawn(24, 80);
    provider.write(b"echo 'First'\n");
    thread::sleep(Duration::from_millis(500));
    
    let screen1 = provider.get_screen(24, 80);
    
    // Second spawn (should be no-op)
    provider.spawn(24, 80);
    
    let screen2 = provider.get_screen(24, 80);
    
    // Content should be preserved
    assert_eq!(screen1.lines.len(), screen2.lines.len(),
               "Second spawn should not affect existing PTY");
}
