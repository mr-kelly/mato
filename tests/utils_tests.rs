use mato::utils::{get_log_path, get_pid_path, get_socket_path, new_id};

#[test]
fn test_id_generation() {
    let id1 = new_id();
    let id2 = new_id();

    // IDs should be non-empty
    assert!(!id1.is_empty());
    assert!(!id2.is_empty());

    // IDs should be different
    assert_ne!(id1, id2);
}

#[test]
fn test_path_generation() {
    let socket = get_socket_path();
    let log = get_log_path();
    let pid = get_pid_path();

    // Paths should end with correct filenames
    assert!(socket.to_string_lossy().ends_with("daemon.sock"));
    assert!(log.to_string_lossy().ends_with("daemon.log"));
    assert!(pid.to_string_lossy().ends_with("daemon.pid"));

    // All paths should be in the same directory
    assert_eq!(socket.parent(), log.parent());
    assert_eq!(log.parent(), pid.parent());
}

#[test]
fn test_paths_contain_mato() {
    let socket = get_socket_path();
    let path_str = socket.to_string_lossy();

    // Path should contain "mato"
    assert!(path_str.contains("mato"));
}

#[test]
fn new_id_produces_unique_ids_under_load() {
    use std::collections::HashSet;
    let ids: HashSet<String> = (0..1000).map(|_| new_id()).collect();
    assert_eq!(ids.len(), 1000, "all 1000 IDs must be unique");
}

#[test]
fn new_id_is_hex_alphanumeric() {
    let id = new_id();
    assert!(
        id.chars().all(|c| c.is_ascii_hexdigit()),
        "id should be all hex digits, got: {id:?}"
    );
}
