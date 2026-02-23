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

use mato::terminal_provider::{CursorShape, ScreenContent};

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
