use mato::protocol::{ClientMsg, ServerMsg};

#[test]
fn test_protocol_serialization() {
    // Test ClientMsg serialization
    let msg = ClientMsg::Spawn {
        tab_id: "test123".to_string(),
        rows: 24,
        cols: 80,
        cwd: None,
        shell: None,
        env: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ClientMsg = serde_json::from_str(&json).unwrap();

    match deserialized {
        ClientMsg::Spawn {
            tab_id, rows, cols, ..
        } => {
            assert_eq!(tab_id, "test123");
            assert_eq!(rows, 24);
            assert_eq!(cols, 80);
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_server_msg_serialization() {
    let msg = ServerMsg::Welcome {
        version: "0.1.0".to_string(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ServerMsg = serde_json::from_str(&json).unwrap();

    match deserialized {
        ServerMsg::Welcome { version } => {
            assert_eq!(version, "0.1.0");
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_input_msg() {
    let msg = ClientMsg::Input {
        tab_id: "abc".to_string(),
        data: vec![65, 66, 67], // "ABC"
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("Input"));
    assert!(json.contains("abc"));
}

#[test]
fn test_paste_msg() {
    let msg = ClientMsg::Paste {
        tab_id: "abc".to_string(),
        data: "line1\nline2".to_string(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ClientMsg = serde_json::from_str(&json).unwrap();

    match deserialized {
        ClientMsg::Paste { tab_id, data } => {
            assert_eq!(tab_id, "abc");
            assert_eq!(data, "line1\nline2");
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_resize_msg() {
    let msg = ClientMsg::Resize {
        tab_id: "test".to_string(),
        rows: 30,
        cols: 100,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ClientMsg = serde_json::from_str(&json).unwrap();

    match deserialized {
        ClientMsg::Resize { rows, cols, .. } => {
            assert_eq!(rows, 30);
            assert_eq!(cols, 100);
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_get_input_modes_msg() {
    let msg = ClientMsg::GetInputModes {
        tab_id: "tab-1".to_string(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ClientMsg = serde_json::from_str(&json).unwrap();

    match deserialized {
        ClientMsg::GetInputModes { tab_id } => assert_eq!(tab_id, "tab-1"),
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_input_modes_server_msg() {
    let msg = ServerMsg::InputModes {
        mouse: true,
        bracketed_paste: false,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ServerMsg = serde_json::from_str(&json).unwrap();

    match deserialized {
        ServerMsg::InputModes {
            mouse,
            bracketed_paste,
        } => {
            assert!(mouse);
            assert!(!bracketed_paste);
        }
        _ => panic!("Wrong message type"),
    }
}

use mato::terminal_provider::{CursorShape, ScreenContent, ScreenLine};

#[test]
fn screen_content_json_roundtrip_preserves_bell_and_focus_events() {
    let content = ScreenContent {
        lines: vec![],
        cursor: (3, 7),
        title: Some("vim".into()),
        cursor_shape: CursorShape::Beam,
        bell: true,
        focus_events_enabled: true,
    };
    let json = serde_json::to_string(&content).unwrap();
    let restored: ScreenContent = serde_json::from_str(&json).unwrap();
    assert!(restored.bell, "bell must survive JSON roundtrip");
    assert!(restored.focus_events_enabled, "focus_events_enabled must survive JSON roundtrip");
    assert_eq!(restored.cursor, (3, 7));
    assert_eq!(restored.title.as_deref(), Some("vim"));
}

#[test]
fn screen_content_msgpack_roundtrip_preserves_bell_and_focus_events() {
    let content = ScreenContent {
        lines: vec![],
        cursor: (1, 2),
        title: None,
        cursor_shape: CursorShape::Block,
        bell: true,
        focus_events_enabled: true,
    };
    let bin = rmp_serde::to_vec(&content).unwrap();
    let restored: ScreenContent = rmp_serde::from_slice(&bin).unwrap();
    assert!(restored.bell);
    assert!(restored.focus_events_enabled);
}

#[test]
fn screen_content_missing_new_fields_deserialize_as_false() {
    // Old JSON without bell/focus_events_enabled should deserialize cleanly (serde default).
    let old_json = r#"{"lines":[],"cursor":[0,0],"title":null,"cursor_shape":"Block"}"#;
    let content: ScreenContent = serde_json::from_str(old_json).unwrap();
    assert!(!content.bell, "old client JSON with no bell field must default to false");
    assert!(!content.focus_events_enabled, "old client JSON with no focus_events_enabled must default to false");
}

#[test]
fn screen_diff_msgpack_roundtrip_preserves_focus_events_enabled() {
    use mato::protocol::ServerMsg;
    use mato::terminal_provider::ScreenLine;
    let msg = ServerMsg::ScreenDiff {
        changed_lines: vec![],
        cursor: (0, 0),
        cursor_shape: CursorShape::Block,
        title: None,
        bell: false,
        focus_events_enabled: true,
    };
    let bin = rmp_serde::to_vec(&msg).unwrap();
    let restored: ServerMsg = rmp_serde::from_slice(&bin).unwrap();
    match restored {
        ServerMsg::ScreenDiff { focus_events_enabled, .. } => {
            assert!(focus_events_enabled, "focus_events_enabled must survive msgpack roundtrip");
        }
        _ => panic!("Expected ScreenDiff"),
    }
}

// ── Additional protocol tests ─────────────────────────────────────────────────

/// Scroll message serializes and deserializes correctly with positive and negative deltas.
#[test]
fn scroll_message_roundtrip() {
    for delta in [-3i32, 0, 5, i32::MAX, i32::MIN] {
        let msg = ClientMsg::Scroll {
            tab_id: "t1".into(),
            delta,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ClientMsg = serde_json::from_str(&json).unwrap();
        match back {
            ClientMsg::Scroll { tab_id, delta: d } => {
                assert_eq!(tab_id, "t1");
                assert_eq!(d, delta);
            }
            _ => panic!("wrong variant"),
        }
    }
}

/// Subscribe message carries rows/cols correctly.
#[test]
fn subscribe_message_roundtrip() {
    let msg = ClientMsg::Subscribe {
        tab_id: "tab-abc".into(),
        rows: 40,
        cols: 160,
    };
    let bin = rmp_serde::to_vec(&msg).unwrap();
    let back: ClientMsg = rmp_serde::from_slice(&bin).unwrap();
    match back {
        ClientMsg::Subscribe { tab_id, rows, cols } => {
            assert_eq!(tab_id, "tab-abc");
            assert_eq!(rows, 40);
            assert_eq!(cols, 160);
        }
        _ => panic!("wrong variant"),
    }
}

/// ScreenDiff with both bell=true and focus_events_enabled=true survives roundtrip.
#[test]
fn screen_diff_bell_and_focus_events_both_true() {
    use mato::protocol::ServerMsg;
    let msg = ServerMsg::ScreenDiff {
        changed_lines: vec![],
        cursor: (5, 10),
        cursor_shape: CursorShape::Beam,
        title: Some("mytitle".into()),
        bell: true,
        focus_events_enabled: true,
    };
    let bin = rmp_serde::to_vec(&msg).unwrap();
    let back: ServerMsg = rmp_serde::from_slice(&bin).unwrap();
    match back {
        ServerMsg::ScreenDiff { bell, focus_events_enabled, cursor, title, cursor_shape, .. } => {
            assert!(bell);
            assert!(focus_events_enabled);
            assert_eq!(cursor, (5, 10));
            assert_eq!(title.as_deref(), Some("mytitle"));
            assert!(matches!(cursor_shape, CursorShape::Beam));
        }
        _ => panic!("wrong variant"),
    }
}

/// InputModes with all combinations roundtrips correctly.
#[test]
fn input_modes_roundtrip_all_combinations() {
    use mato::protocol::ServerMsg;
    for (mouse, paste) in [(false, false), (true, false), (false, true), (true, true)] {
        let msg = ServerMsg::InputModes { mouse, bracketed_paste: paste };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ServerMsg = serde_json::from_str(&json).unwrap();
        match back {
            ServerMsg::InputModes { mouse: m, bracketed_paste: p } => {
                assert_eq!(m, mouse);
                assert_eq!(p, paste);
            }
            _ => panic!("expected InputModes"),
        }
    }
}

/// Screen full message with changed lines roundtrips via msgpack.
#[test]
fn screen_full_with_changed_lines_msgpack_roundtrip() {
    use mato::protocol::ServerMsg;
    use mato::terminal_provider::ScreenCell;
    let cell = ScreenCell {
        ch: 'A',
        display_width: 1,
        fg: None, bg: None,
        bold: true, italic: false, underline: false,
        dim: false, reverse: false, strikethrough: false, hidden: false,
        underline_color: None, zerowidth: None,
    };
    let line = ScreenLine { cells: vec![cell] };
    let content = ScreenContent {
        lines: vec![line],
        cursor: (0, 0),
        title: Some("test".into()),
        cursor_shape: CursorShape::Block,
        bell: false,
        focus_events_enabled: false,
    };
    let msg = ServerMsg::Screen { tab_id: "t".into(), content };
    let bin = rmp_serde::to_vec(&msg).unwrap();
    let back: ServerMsg = rmp_serde::from_slice(&bin).unwrap();
    match back {
        ServerMsg::Screen { tab_id, content: c } => {
            assert_eq!(tab_id, "t");
            assert_eq!(c.lines.len(), 1);
            assert_eq!(c.lines[0].cells[0].ch, 'A');
            assert!(c.lines[0].cells[0].bold);
        }
        _ => panic!("expected Screen"),
    }
}

/// Error message roundtrip.
#[test]
fn error_message_roundtrip() {
    use mato::protocol::ServerMsg;
    let msg = ServerMsg::Error { message: "tab not found".into() };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ServerMsg = serde_json::from_str(&json).unwrap();
    match back {
        ServerMsg::Error { message } => assert_eq!(message, "tab not found"),
        _ => panic!("expected Error"),
    }
}

/// GetInputModes request roundtrip.
#[test]
fn get_input_modes_request_roundtrip() {
    let msg = ClientMsg::GetInputModes { tab_id: "abc".into() };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ClientMsg = serde_json::from_str(&json).unwrap();
    match back {
        ClientMsg::GetInputModes { tab_id } => assert_eq!(tab_id, "abc"),
        _ => panic!("expected GetInputModes"),
    }
}

/// ScreenDiff with changed lines preserves line content.
#[test]
fn screen_diff_changed_lines_preserve_cell_attributes() {
    use mato::protocol::ServerMsg;
    use mato::terminal_provider::ScreenCell;
    let cell = ScreenCell {
        ch: '✓',
        display_width: 1,
        fg: None, bg: None,
        bold: false, italic: true, underline: true,
        dim: false, reverse: true, strikethrough: false, hidden: false,
        underline_color: None, zerowidth: None,
    };
    let line = ScreenLine { cells: vec![cell] };
    let msg = ServerMsg::ScreenDiff {
        changed_lines: vec![(3u16, line)],
        cursor: (3, 1),
        cursor_shape: CursorShape::Underline,
        title: None,
        bell: false,
        focus_events_enabled: false,
    };
    let bin = rmp_serde::to_vec(&msg).unwrap();
    let back: ServerMsg = rmp_serde::from_slice(&bin).unwrap();
    match back {
        ServerMsg::ScreenDiff { changed_lines, cursor, cursor_shape, .. } => {
            assert_eq!(cursor, (3, 1));
            assert!(matches!(cursor_shape, CursorShape::Underline));
            assert_eq!(changed_lines.len(), 1);
            let (idx, restored_line) = &changed_lines[0];
            assert_eq!(*idx, 3);
            let c = &restored_line.cells[0];
            assert_eq!(c.ch, '✓');
            assert!(c.italic);
            assert!(c.underline);
            assert!(c.reverse);
        }
        _ => panic!("expected ScreenDiff"),
    }
}
