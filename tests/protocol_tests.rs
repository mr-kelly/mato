use mato::protocol::{ClientMsg, ServerMsg};

#[test]
fn test_protocol_serialization() {
    // Test ClientMsg serialization
    let msg = ClientMsg::Spawn {
        tab_id: "test123".to_string(),
        rows: 24,
        cols: 80,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ClientMsg = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        ClientMsg::Spawn { tab_id, rows, cols } => {
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
