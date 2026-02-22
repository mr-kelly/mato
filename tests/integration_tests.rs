/// Integration tests: spin up handle_client in-process via a real Unix socket,
/// verify the full daemon protocol round-trip without forking.
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use parking_lot::Mutex;

use mato::config::Config;
use mato::daemon::daemon::handle_client;
use mato::protocol::{ClientMsg, ServerMsg};
use mato::providers::PtyProvider;

fn start_daemon(socket_path: &str) -> Arc<DashMap<String, Arc<Mutex<PtyProvider>>>> {
    let tabs: Arc<DashMap<String, Arc<Mutex<PtyProvider>>>> = Arc::new(DashMap::new());
    let tabs_clone = tabs.clone();
    let path_thread = socket_path.to_string();
    let path_wait = socket_path.to_string();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let _ = std::fs::remove_file(&path_thread);
            let listener = tokio::net::UnixListener::bind(&path_thread).unwrap();
            loop {
                if let Ok((stream, _)) = listener.accept().await {
                    let tabs = tabs_clone.clone();
                    let config = Arc::new(Mutex::new(Config::default()));
                    tokio::spawn(async move {
                        let latest_version = Arc::new(parking_lot::Mutex::new(None));
                        let _ = handle_client(stream, tabs, config, 1, latest_version).await;
                    });
                }
            }
        });
    });

    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(20));
        if UnixStream::connect(&path_wait).is_ok() {
            break;
        }
    }
    tabs
}

fn send_recv(socket_path: &str, msg: &ClientMsg) -> ServerMsg {
    let mut stream = UnixStream::connect(socket_path).expect("connect");
    let json = serde_json::to_vec(msg).unwrap();
    stream.write_all(&json).unwrap();
    stream.write_all(b"\n").unwrap();
    stream.flush().unwrap();
    let mut line = String::new();
    BufReader::new(&stream).read_line(&mut line).unwrap();
    serde_json::from_str(&line).expect("parse response")
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn daemon_hello_returns_welcome() {
    let socket = "/tmp/mato_test_hello.sock";
    start_daemon(socket);
    let resp = send_recv(
        socket,
        &ClientMsg::Hello {
            version: "test".into(),
        },
    );
    assert!(matches!(resp, ServerMsg::Welcome { .. }));
}

#[test]
fn daemon_spawn_creates_tab() {
    let socket = "/tmp/mato_test_spawn.sock";
    let tabs = start_daemon(socket);
    send_recv(
        socket,
        &ClientMsg::Spawn {
            tab_id: "tab-1".into(),
            rows: 24,
            cols: 80,
        },
    );
    std::thread::sleep(Duration::from_millis(100));
    assert!(tabs.contains_key("tab-1"));
}

#[test]
fn daemon_get_screen_returns_screen() {
    let socket = "/tmp/mato_test_screen.sock";
    start_daemon(socket);
    send_recv(
        socket,
        &ClientMsg::Spawn {
            tab_id: "tab-s".into(),
            rows: 24,
            cols: 80,
        },
    );
    std::thread::sleep(Duration::from_millis(100));
    let resp = send_recv(
        socket,
        &ClientMsg::GetScreen {
            tab_id: "tab-s".into(),
            rows: 24,
            cols: 80,
        },
    );
    assert!(
        matches!(resp, ServerMsg::Screen { .. }),
        "expected Screen, got {:?}",
        resp
    );
}

#[test]
fn daemon_get_screen_unknown_tab_returns_error() {
    let socket = "/tmp/mato_test_err.sock";
    start_daemon(socket);
    let resp = send_recv(
        socket,
        &ClientMsg::GetScreen {
            tab_id: "no-such-tab".into(),
            rows: 24,
            cols: 80,
        },
    );
    assert!(matches!(resp, ServerMsg::Error { .. }));
}

#[test]
fn daemon_get_idle_status_includes_spawned_tab() {
    let socket = "/tmp/mato_test_idle.sock";
    start_daemon(socket);
    send_recv(
        socket,
        &ClientMsg::Spawn {
            tab_id: "tab-idle".into(),
            rows: 24,
            cols: 80,
        },
    );
    std::thread::sleep(Duration::from_millis(100));
    let resp = send_recv(socket, &ClientMsg::GetIdleStatus);
    match resp {
        ServerMsg::IdleStatus { tabs } => {
            let ids: Vec<&str> = tabs.iter().map(|(id, _)| id.as_str()).collect();
            assert!(
                ids.contains(&"tab-idle"),
                "idle status should include spawned tab, got: {:?}",
                ids
            );
        }
        other => panic!("expected IdleStatus, got {:?}", other),
    }
}

#[test]
fn daemon_spawn_twice_is_idempotent() {
    let socket = "/tmp/mato_test_idem.sock";
    let tabs = start_daemon(socket);
    for _ in 0..2 {
        send_recv(
            socket,
            &ClientMsg::Spawn {
                tab_id: "tab-dup".into(),
                rows: 24,
                cols: 80,
            },
        );
    }
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(tabs.iter().filter(|e| e.key() == "tab-dup").count(), 1);
}
