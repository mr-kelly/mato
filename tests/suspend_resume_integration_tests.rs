#![cfg(unix)]

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};

fn find_mato_bin() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_mato") {
        let p = PathBuf::from(path);
        if p.exists() {
            return p;
        }
    }

    let exe = std::env::current_exe().expect("current_exe");
    let debug_dir = exe
        .parent()
        .and_then(|p| p.parent())
        .expect("target/debug dir");
    let candidate = debug_dir.join("mato");
    assert!(
        candidate.exists(),
        "mato binary not found at {}",
        candidate.display()
    );
    candidate
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_minimal_state(config_home: &Path) {
    let state_dir = config_home.join("mato");
    fs::create_dir_all(&state_dir).expect("create config dir");
    let state = r#"{
  "current_office": 0,
  "offices": [
    {
      "id": "office-1",
      "name": "Default",
      "active_desk": 0,
      "desks": [
        {
          "id": "desk-1",
          "name": "Work",
          "active_tab": 0,
          "tabs": [
            { "id": "tab-1", "name": "shell" }
          ]
        }
      ]
    }
  ]
}"#;
    fs::write(state_dir.join("state.json"), state).expect("write state.json");
}

fn kill_daemon_if_present(state_home: &Path) {
    let pid_path = state_home.join("mato").join("daemon.pid");
    let Ok(text) = fs::read_to_string(pid_path) else {
        return;
    };
    let Ok(pid) = text.trim().parse::<i32>() else {
        return;
    };
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }
}

#[test]
#[ignore] // Run with: cargo test --test suspend_resume_integration_tests -- --ignored
fn client_survives_suspend_and_resume_signals() {
    let bin_path = find_mato_bin();

    let test_root = unique_temp_dir("mato-suspend-resume");
    let home = test_root.join("home");
    let config_home = test_root.join("config");
    let state_home = test_root.join("state");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&config_home).expect("create config home");
    fs::create_dir_all(&state_home).expect("create state home");
    write_minimal_state(&config_home);

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("openpty");

    let mut cmd = CommandBuilder::new(bin_path);
    cmd.env("HOME", home);
    cmd.env("XDG_CONFIG_HOME", &config_home);
    cmd.env("XDG_STATE_HOME", &state_home);
    cmd.env("TERM", "xterm-256color");

    let mut child = pair.slave.spawn_command(cmd).expect("spawn mato");
    drop(pair.slave);

    // Drain PTY output so the app doesn't block on a full PTY buffer.
    let mut reader = pair.master.try_clone_reader().expect("clone reader");
    let _drain = thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
        }
    });

    thread::sleep(Duration::from_millis(800));

    let pid = child.process_id().expect("child pid") as i32;
    unsafe {
        assert_eq!(libc::kill(pid, libc::SIGTSTP), 0, "send SIGTSTP");
    }
    thread::sleep(Duration::from_millis(300));
    unsafe {
        assert_eq!(libc::kill(pid, libc::SIGCONT), 0, "send SIGCONT");
    }
    thread::sleep(Duration::from_millis(500));

    let still_alive = unsafe { libc::kill(pid, 0) == 0 };
    assert!(
        still_alive,
        "client should still be alive after SIGTSTP/SIGCONT"
    );

    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }
    let _status = child.wait().expect("wait child");
    let alive_after_wait = unsafe { libc::kill(pid, 0) == 0 };
    assert!(!alive_after_wait, "client should exit after SIGTERM");

    kill_daemon_if_present(&state_home);
}
