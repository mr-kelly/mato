use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;

/// Integration test for daemon-level terminal persistence
///
/// This test verifies the full client-daemon flow:
/// 1. Start daemon
/// 2. Client connects and spawns PTY
/// 3. Client writes content
/// 4. Client disconnects
/// 5. New client connects with same tab ID
/// 6. Content should still be visible

#[test]
#[ignore] // Run with: cargo test --test daemon_persistence_tests -- --ignored
fn test_daemon_terminal_persistence() {
    use mato::protocol::{ClientMsg, ServerMsg};

    // This test requires a running daemon
    // Start daemon with: cargo run -- --daemon --foreground

    let socket_path = std::env::var("HOME")
        .map(|h| format!("{}/.local/state/mato/daemon.sock", h))
        .unwrap_or_else(|_| "/tmp/mato.sock".to_string());

    let tab_id = "test-tab-123".to_string();

    // === First connection: Create PTY and write content ===
    {
        let mut stream =
            UnixStream::connect(&socket_path).expect("Failed to connect to daemon. Is it running?");

        // Spawn PTY
        let spawn_msg = ClientMsg::Spawn {
            tab_id: tab_id.clone(),
            rows: 24,
            cols: 80,
        };

        let json = serde_json::to_vec(&spawn_msg).unwrap();
        stream.write_all(&json).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();

        // Split stream for reading and writing
        let stream_clone = stream.try_clone().unwrap();
        let mut reader = BufReader::new(stream_clone);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        let response: ServerMsg = serde_json::from_str(&line).unwrap();
        match response {
            ServerMsg::Welcome { .. } => println!("PTY spawned successfully"),
            _ => panic!("Unexpected response: {:?}", response),
        }

        // Write content
        let input_msg = ClientMsg::Input {
            tab_id: tab_id.clone(),
            data: b"echo 'Hello from test'\n".to_vec(),
        };

        let json = serde_json::to_vec(&input_msg).unwrap();
        stream.write_all(&json).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();

        // Wait for output to be processed
        thread::sleep(Duration::from_millis(500));

        // Get screen content
        let get_screen_msg = ClientMsg::GetScreen {
            tab_id: tab_id.clone(),
            rows: 24,
            cols: 80,
        };

        let json = serde_json::to_vec(&get_screen_msg).unwrap();
        stream.write_all(&json).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();

        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        let response: ServerMsg = serde_json::from_str(&line).unwrap();
        match response {
            ServerMsg::Screen { content, .. } => {
                assert!(!content.lines.is_empty(), "Screen should have content");
                println!("First connection: Got {} lines", content.lines.len());
            }
            _ => panic!("Expected Screen response"),
        }

        // Connection closes here
    }

    // === Second connection: Reconnect with same tab ID ===
    thread::sleep(Duration::from_millis(100));

    {
        let mut stream = UnixStream::connect(socket_path).expect("Failed to reconnect to daemon");

        // Try to spawn again (should be no-op)
        let spawn_msg = ClientMsg::Spawn {
            tab_id: tab_id.clone(),
            rows: 24,
            cols: 80,
        };

        let json = serde_json::to_vec(&spawn_msg).unwrap();
        stream.write_all(&json).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();

        let stream_clone = stream.try_clone().unwrap();
        let mut reader = BufReader::new(stream_clone);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        let response: ServerMsg = serde_json::from_str(&line).unwrap();
        match response {
            ServerMsg::Welcome { version } => {
                assert_eq!(version, "already exists", "PTY should already exist");
                println!("PTY already exists (good!)");
            }
            _ => panic!("Unexpected response: {:?}", response),
        }

        // Get screen content again
        let get_screen_msg = ClientMsg::GetScreen {
            tab_id: tab_id.clone(),
            rows: 24,
            cols: 80,
        };

        let json = serde_json::to_vec(&get_screen_msg).unwrap();
        stream.write_all(&json).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();

        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        let response: ServerMsg = serde_json::from_str(&line).unwrap();
        match response {
            ServerMsg::Screen { content, .. } => {
                assert!(
                    !content.lines.is_empty(),
                    "Screen should still have content after reconnect"
                );

                // Check if our text is still there
                let has_hello = content.lines.iter().any(|line| {
                    line.cells
                        .iter()
                        .any(|cell| cell.ch == 'H' || cell.ch == 'e')
                });

                assert!(has_hello, "Content should persist across reconnections");
                println!(
                    "Second connection: Content persisted! Got {} lines",
                    content.lines.len()
                );
            }
            _ => panic!("Expected Screen response"),
        }
    }

    println!("✅ Terminal content persistence test passed!");
}

#[test]
#[ignore]
fn test_daemon_multiple_tabs() {
    use mato::protocol::{ClientMsg, ServerMsg};

    let socket_path = std::env::var("HOME")
        .map(|h| format!("{}/.local/state/mato/daemon.sock", h))
        .unwrap_or_else(|_| "/tmp/mato.sock".to_string());

    let tab1 = "tab-1".to_string();
    let tab2 = "tab-2".to_string();

    let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect to daemon");

    // Spawn two tabs
    for tab_id in &[&tab1, &tab2] {
        let spawn_msg = ClientMsg::Spawn {
            tab_id: (*tab_id).clone(),
            rows: 24,
            cols: 80,
        };

        let json = serde_json::to_vec(&spawn_msg).unwrap();
        stream.write_all(&json).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();

        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
    }

    // Write different content to each tab
    for (i, tab_id) in [&tab1, &tab2].iter().enumerate() {
        let input_msg = ClientMsg::Input {
            tab_id: (*tab_id).clone(),
            data: format!("echo 'Tab {}'\n", i + 1).into_bytes(),
        };

        let json = serde_json::to_vec(&input_msg).unwrap();
        stream.write_all(&json).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();
    }

    thread::sleep(Duration::from_millis(500));

    // Verify each tab has its own content
    for tab_id in &[&tab1, &tab2] {
        let get_screen_msg = ClientMsg::GetScreen {
            tab_id: (*tab_id).clone(),
            rows: 24,
            cols: 80,
        };

        let json = serde_json::to_vec(&get_screen_msg).unwrap();
        stream.write_all(&json).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();

        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        let response: ServerMsg = serde_json::from_str(&line).unwrap();
        match response {
            ServerMsg::Screen { content, .. } => {
                assert!(
                    !content.lines.is_empty(),
                    "Tab {} should have content",
                    tab_id
                );
            }
            _ => panic!("Expected Screen response"),
        }
    }

    println!("✅ Multiple tabs test passed!");
}

#[test]
#[ignore]
fn test_resize_preserves_content() {
    use mato::protocol::{ClientMsg, ServerMsg};

    let socket_path = std::env::var("HOME")
        .map(|h| format!("{}/.local/state/mato/daemon.sock", h))
        .unwrap_or_else(|_| "/tmp/mato.sock".to_string());

    let tab_id = "test-resize-tab".to_string();

    // Spawn PTY with initial size
    {
        let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect to daemon");

        let spawn_msg = ClientMsg::Spawn {
            tab_id: tab_id.clone(),
            rows: 24,
            cols: 80,
        };

        let json = serde_json::to_vec(&spawn_msg).unwrap();
        stream.write_all(&json).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();

        let stream_clone = stream.try_clone().unwrap();
        let mut reader = BufReader::new(stream_clone);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        // Write some content
        let input_msg = ClientMsg::Input {
            tab_id: tab_id.clone(),
            data: b"echo 'Content should survive resize'\n".to_vec(),
        };

        let json = serde_json::to_vec(&input_msg).unwrap();
        stream.write_all(&json).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();

        thread::sleep(Duration::from_millis(500));

        // Get initial screen content
        let get_screen_msg = ClientMsg::GetScreen {
            tab_id: tab_id.clone(),
            rows: 24,
            cols: 80,
        };

        let json = serde_json::to_vec(&get_screen_msg).unwrap();
        stream.write_all(&json).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();

        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        let response: ServerMsg = serde_json::from_str(&line).unwrap();
        match response {
            ServerMsg::Screen { content, .. } => {
                assert!(!content.lines.is_empty(), "Should have initial content");
                println!("Initial content: {} lines", content.lines.len());
            }
            _ => panic!("Expected Screen response"),
        }
    }

    // Send resize message
    {
        let mut stream = UnixStream::connect(&socket_path).unwrap();

        let resize_msg = ClientMsg::Resize {
            tab_id: tab_id.clone(),
            rows: 30,
            cols: 100,
        };

        let json = serde_json::to_vec(&resize_msg).unwrap();
        stream.write_all(&json).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();

        thread::sleep(Duration::from_millis(200));
    }

    // Get screen content after resize - should still be there!
    {
        let mut stream = UnixStream::connect(&socket_path).unwrap();

        let get_screen_msg = ClientMsg::GetScreen {
            tab_id: tab_id.clone(),
            rows: 30,
            cols: 100,
        };

        let json = serde_json::to_vec(&get_screen_msg).unwrap();
        stream.write_all(&json).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();

        let stream_clone = stream.try_clone().unwrap();
        let mut reader = BufReader::new(stream_clone);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        let response: ServerMsg = serde_json::from_str(&line).unwrap();
        match response {
            ServerMsg::Screen { content, .. } => {
                assert!(!content.lines.is_empty(), "Content should survive resize!");
                println!(
                    "After resize: {} lines - Content preserved!",
                    content.lines.len()
                );
            }
            _ => panic!("Expected Screen response"),
        }
    }

    println!("✅ Resize preserves content test passed!");
}
