use std::io;
use std::path::Path;
use std::time::Duration;

pub fn run_daemon(foreground: bool) -> io::Result<()> {
    let socket_path = crate::utils::get_socket_path();
    let log_path = crate::utils::get_log_path();
    let lock_path = crate::utils::get_lock_path();
    let pid_path = crate::utils::get_pid_path();

    // Acquire lock file BEFORE forking to prevent race conditions
    let _lock = super::DaemonLock::acquire(lock_path).map_err(|e| {
        eprintln!("Failed to acquire daemon lock: {}", e);
        eprintln!("Is another daemon already running?");
        e
    })?;

    if !foreground {
        // Fork to background
        unsafe {
            let pid = libc::fork();
            if pid < 0 {
                return Err(io::Error::last_os_error());
            }
            if pid > 0 {
                // Parent: print info and exit
                println!("Daemon started in background");
                println!("Socket: {}", socket_path.display());
                println!("Log: {}", log_path.display());
                std::process::exit(0);
            }
            // Child continues
            libc::setsid();
        }
    }

    // Create PID file after fork (so it has the correct PID)
    let _pid_file = super::PidFile::create(pid_path)?;

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    // Log to file (background mode) or console + file (foreground mode)
    if foreground {
        use tracing_subscriber::fmt::writer::MakeWriterExt;
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr.and(log_file))
            .with_ansi(false)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_writer(log_file)
            .with_ansi(false)
            .init();
    }

    tracing::info!("========== DAEMON STARTING ==========");
    tracing::info!("Socket: {}", socket_path.display());
    tracing::info!("Log: {}", log_path.display());
    tracing::info!("PID: {}", std::process::id());
    tracing::info!(
        "Start time: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    let daemon = super::Daemon::new();
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { daemon.run(&socket_path.to_string_lossy()).await })
        .map_err(io::Error::other)
}

pub fn ensure_daemon_running() -> io::Result<()> {
    use std::os::unix::net::UnixStream;

    let socket_path = crate::utils::get_socket_path();

    // Try to connect
    if UnixStream::connect(&socket_path).is_ok() {
        return Ok(()); // Already running
    }

    // Remove stale socket
    let _ = std::fs::remove_file(&socket_path);

    // Spawn daemon
    std::process::Command::new(std::env::current_exe()?)
        .arg("--daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    // Wait for daemon to be ready
    for _ in 0..20 {
        std::thread::sleep(Duration::from_millis(50));
        if UnixStream::connect(&socket_path).is_ok() {
            return Ok(());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "Daemon failed to start",
    ))
}

pub fn kill_all() -> io::Result<()> {
    let socket_path = crate::utils::get_socket_path();
    let pid_path = crate::utils::get_pid_path();
    let lock_path = crate::utils::get_lock_path();

    let self_pid = std::process::id();
    let mut daemon_pid: Option<u32> = None;
    let mut tab_processes_killed = 0usize;

    if let Some(pid) = super::PidFile::read(&pid_path) {
        daemon_pid = Some(pid);
        if process_exists(pid) {
            let descendants = collect_descendants(pid);
            tab_processes_killed = descendants.len();

            #[cfg(unix)]
            unsafe {
                // Best-effort whole process-group termination (daemon is setsid leader).
                libc::kill(-(pid as libc::pid_t), libc::SIGTERM);
                libc::kill(pid as libc::pid_t, libc::SIGTERM);
            }

            // Wait up to 2s for graceful exit.
            for _ in 0..20 {
                std::thread::sleep(Duration::from_millis(100));
                let any_child_alive = !collect_descendants(pid).is_empty();
                if !process_exists(pid) && !any_child_alive {
                    break;
                }
            }

            // Force kill any remaining daemon or child processes.
            if process_exists(pid) || !collect_descendants(pid).is_empty() {
                #[cfg(unix)]
                unsafe {
                    libc::kill(-(pid as libc::pid_t), libc::SIGKILL);
                    let descendants = collect_descendants(pid);
                    tab_processes_killed = tab_processes_killed.max(descendants.len());
                    for child in &descendants {
                        if process_exists(*child) {
                            libc::kill(*child as libc::pid_t, libc::SIGKILL);
                        }
                    }
                    libc::kill(pid as libc::pid_t, libc::SIGKILL);
                }
            }
        }
    }

    // Kill all mato client processes (same executable, not daemon, not self).
    let current_exe = std::env::current_exe().ok();
    let mut clients_killed = 0usize;
    if let Some(exe) = current_exe.as_ref() {
        clients_killed = kill_client_processes(exe, self_pid, daemon_pid);
    }

    // Cleanup stale files regardless of daemon exit path.
    let _ = std::fs::remove_file(&socket_path);
    let _ = std::fs::remove_file(&pid_path);
    let _ = std::fs::remove_file(&lock_path);

    println!(
        "Stopped daemon, {} client process(es), {} tab process(es).",
        clients_killed, tab_processes_killed
    );
    Ok(())
}

fn process_exists(pid: u32) -> bool {
    Path::new(&format!("/proc/{}", pid)).exists()
}

fn kill_client_processes(exe: &Path, self_pid: u32, daemon_pid: Option<u32>) -> usize {
    let mut targets = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return 0;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(pid_str) = name.to_str() else {
            continue;
        };
        let Ok(pid) = pid_str.parse::<u32>() else {
            continue;
        };
        if pid == self_pid || daemon_pid == Some(pid) {
            continue;
        }

        let proc_exe = entry.path().join("exe");
        let Ok(link) = std::fs::read_link(proc_exe) else {
            continue;
        };
        if !same_executable(exe, &link) {
            continue;
        }

        let cmdline_path = entry.path().join("cmdline");
        let cmdline = std::fs::read(cmdline_path).unwrap_or_default();
        if cmdline.windows(b"--daemon".len()).any(|w| w == b"--daemon") {
            continue;
        }
        if cmdline.windows(b"--kill".len()).any(|w| w == b"--kill") {
            continue;
        }

        targets.push(pid);
    }

    #[cfg(unix)]
    unsafe {
        for pid in &targets {
            libc::kill(*pid as libc::pid_t, libc::SIGTERM);
        }
    }

    // Wait up to 1s, then hard-kill survivors.
    for _ in 0..10 {
        std::thread::sleep(Duration::from_millis(100));
        if targets.iter().all(|p| !process_exists(*p)) {
            break;
        }
    }

    #[cfg(unix)]
    unsafe {
        for pid in &targets {
            if process_exists(*pid) {
                libc::kill(*pid as libc::pid_t, libc::SIGKILL);
            }
        }
    }

    targets.len()
}

fn same_executable(exe: &Path, link: &Path) -> bool {
    if link == exe {
        return true;
    }
    match (std::fs::canonicalize(exe), std::fs::canonicalize(link)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

fn collect_descendants(root_pid: u32) -> Vec<u32> {
    use std::collections::HashMap;
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };

    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(pid_str) = name.to_str() else {
            continue;
        };
        let Ok(pid) = pid_str.parse::<u32>() else {
            continue;
        };
        let stat_path = entry.path().join("stat");
        let Ok(stat) = std::fs::read_to_string(stat_path) else {
            continue;
        };
        // /proc/<pid>/stat: field 4 is ppid; comm can contain spaces in parentheses.
        let Some(end_comm) = stat.rfind(')') else {
            continue;
        };
        let tail = &stat[end_comm + 1..];
        let parts: Vec<&str> = tail.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let Ok(ppid) = parts[1].parse::<u32>() else {
            continue;
        };
        children.entry(ppid).or_default().push(pid);
    }

    let mut out = Vec::new();
    let mut stack = vec![root_pid];
    while let Some(p) = stack.pop() {
        if let Some(kids) = children.get(&p) {
            for child in kids {
                out.push(*child);
                stack.push(*child);
            }
        }
    }
    out
}
