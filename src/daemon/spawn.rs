use std::io;
use std::time::Duration;

pub fn run_daemon(foreground: bool) -> io::Result<()> {
    let socket_path = crate::utils::get_socket_path();
    let log_path = crate::utils::get_log_path();
    let lock_path = crate::utils::get_lock_path();
    let pid_path = crate::utils::get_pid_path();
    
    // Acquire lock file BEFORE forking to prevent race conditions
    let _lock = super::DaemonLock::acquire(lock_path)
        .map_err(|e| {
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
    tracing::info!("Start time: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
    
    let daemon = super::Daemon::new();
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        daemon.run(&socket_path.to_string_lossy()).await
    }).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
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
    
    Err(io::Error::new(io::ErrorKind::TimedOut, "Daemon failed to start"))
}
