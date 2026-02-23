use crate::emulators::{AlacrittyEmulator, Vt100Emulator};
use crate::passthrough::split_passthrough;
use crate::terminal_emulator::TerminalEmulator;
use crate::terminal_provider::{ScreenContent, TerminalProvider};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::{
    env,
    io::{Read, Write},
    sync::{
        atomic::AtomicUsize,
        Arc, Mutex,
    },
    thread,
    time::Instant,
};

pub struct PtyProvider {
    pty: Option<PtyState>,
    pub last_output: Arc<Mutex<Instant>>,
    current_size: (u16, u16),
    /// Per-tab spawn options (remembered for respawn)
    spawn_cwd: Option<String>,
    spawn_shell: Option<String>,
    spawn_env: Option<Vec<(String, String)>>,
    /// Notified whenever the PTY reader thread processes new output.
    pub output_notify: Arc<tokio::sync::Notify>,
    /// Number of active push-mode subscribers for this tab.
    pub subscriber_count: Arc<AtomicUsize>,
    /// Intercepted APC sequences (Kitty graphics, Sixel, iTerm2) pending delivery to clients.
    pub pending_graphics: Arc<Mutex<Vec<Vec<u8>>>>,
    /// Current working directory from OSC 7 notifications.
    pub current_cwd: Arc<Mutex<Option<String>>>,
}

struct PtyState {
    writer: Box<dyn Write + Send>,
    emulator: Arc<Mutex<Box<dyn TerminalEmulator>>>,
    master: Box<dyn portable_pty::MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl PtyProvider {
    pub fn new() -> Self {
        Self {
            pty: None,
            last_output: Arc::new(Mutex::new(Instant::now())),
            current_size: (24, 80),
            spawn_cwd: None,
            spawn_shell: None,
            spawn_env: None,
            output_notify: Arc::new(tokio::sync::Notify::new()),
            subscriber_count: Arc::new(AtomicUsize::new(0)),
            pending_graphics: Arc::new(Mutex::new(Vec::new())),
            current_cwd: Arc::new(Mutex::new(None)),
        }
    }

    fn create_emulator(rows: u16, cols: u16) -> Box<dyn TerminalEmulator> {
        // Load config
        let config = crate::config::Config::load();

        match config.emulator.as_str() {
            "vt100" => {
                tracing::info!("Using VT100 emulator (from config)");
                Box::new(Vt100Emulator::new(rows, cols))
            }
            _ => {
                tracing::info!("Using Alacritty emulator (from config)");
                Box::new(AlacrittyEmulator::new(rows, cols))
            }
        }
    }

    fn resolve_shell() -> String {
        // Prefer explicit environment from parent process.
        if let Some(shell) = env::var_os("SHELL") {
            let shell = shell.to_string_lossy().trim().to_string();
            if !shell.is_empty() {
                return shell;
            }
        }

        #[cfg(unix)]
        {
            use std::ffi::CStr;
            let shell_ptr = unsafe {
                let uid = libc::getuid();
                let pw = libc::getpwuid(uid);
                if pw.is_null() {
                    std::ptr::null()
                } else {
                    (*pw).pw_shell
                }
            };
            if !shell_ptr.is_null() {
                let shell = unsafe { CStr::from_ptr(shell_ptr) }
                    .to_string_lossy()
                    .trim()
                    .to_string();
                if !shell.is_empty() {
                    return shell;
                }
            }
        }

        "/bin/sh".to_string()
    }

    pub fn child_pid(&self) -> Option<u32> {
        self.pty.as_ref().and_then(|p| p.child.process_id())
    }

    pub fn spawn_with_options(
        &mut self,
        rows: u16,
        cols: u16,
        cwd: Option<&str>,
        shell: Option<&str>,
        env: Option<&[(String, String)]>,
    ) {
        self.spawn_cwd = cwd.map(|s| s.to_string());
        self.spawn_shell = shell.map(|s| s.to_string());
        self.spawn_env = env.map(|e| e.to_vec());
        self.current_size = (rows.max(1), cols.max(1));
        self.ensure_running();
    }

    fn spawn_with_shell(
        rows: u16,
        cols: u16,
        shell: &str,
        cwd: Option<&str>,
        env: Option<&[(String, String)]>,
        last_output: Arc<Mutex<Instant>>,
        output_notify: Arc<tokio::sync::Notify>,
        pending_graphics: Arc<Mutex<Vec<Vec<u8>>>>,
        current_cwd: Arc<Mutex<Option<String>>>,
    ) -> Option<PtyState> {
        let pty_system = native_pty_system();
        let pair = match pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        }) {
            Ok(pair) => pair,
            Err(e) => {
                tracing::error!("Failed to open PTY ({}x{}): {}", rows, cols, e);
                return None;
            }
        };

        tracing::info!("Spawning PTY shell: {}", shell);
        let mut cmd = CommandBuilder::new(shell);
        cmd.env("TERM", "xterm-256color");
        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }
        if let Some(vars) = env {
            for (k, v) in vars {
                cmd.env(k, v);
            }
        }
        let child = match pair.slave.spawn_command(cmd) {
            Ok(child) => child,
            Err(e) => {
                tracing::warn!("Failed to spawn shell '{}': {}", shell, e);
                return None;
            }
        };

        let emulator = Self::create_emulator(rows, cols);
        let emulator = Arc::new(Mutex::new(emulator));
        let emulator_clone = Arc::clone(&emulator);
        let last_output_clone = Arc::clone(&last_output);

        let mut reader = match pair.master.try_clone_reader() {
            Ok(reader) => reader,
            Err(e) => {
                tracing::error!("Failed to clone PTY reader: {}", e);
                return None;
            }
        };
        thread::spawn(move || {
            let mut buf = [0u8; 16384];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let parsed = split_passthrough(&buf[..n]);
                        // Feed normal bytes to emulator
                        if !parsed.normal.is_empty() {
                            emulator_clone.lock().unwrap().process(&parsed.normal);
                        }
                        // Collect APC sequences for graphics passthrough
                        if !parsed.apc_seqs.is_empty() {
                            if let Ok(mut g) = pending_graphics.lock() {
                                g.extend(parsed.apc_seqs);
                            }
                        }
                        // Update working directory from OSC 7
                        if let Some(path) = parsed.osc7_paths.into_iter().last() {
                            if let Ok(mut cwd_guard) = current_cwd.lock() {
                                *cwd_guard = Some(path);
                            }
                        }
                        *last_output_clone.lock().unwrap() = Instant::now();
                        output_notify.notify_waiters();
                    }
                }
            }
        });

        let writer = match pair.master.take_writer() {
            Ok(writer) => writer,
            Err(e) => {
                tracing::error!("Failed to take PTY writer: {}", e);
                return None;
            }
        };

        Some(PtyState {
            writer,
            emulator,
            master: pair.master,
            child,
        })
    }

    fn child_exited(pty: &mut PtyState) -> bool {
        match pty.child.try_wait() {
            Ok(Some(status)) => {
                tracing::info!("PTY child exited with status: {:?}", status);
                true
            }
            Ok(None) => false,
            Err(e) => {
                tracing::warn!("Failed to check PTY child status: {}", e);
                true
            }
        }
    }

    pub fn ensure_running(&mut self) {
        let rows = self.current_size.0.max(1);
        let cols = self.current_size.1.max(1);

        let needs_respawn = match self.pty.as_mut() {
            Some(pty) => Self::child_exited(pty),
            None => true,
        };

        if !needs_respawn {
            return;
        }

        self.pty = None;

        let primary_shell = self.spawn_shell.clone().unwrap_or_else(Self::resolve_shell);
        let fallback_shell = "/bin/sh".to_string();
        let cwd = self.spawn_cwd.as_deref();
        let env = self.spawn_env.as_deref();
        let mut spawned = Self::spawn_with_shell(
            rows,
            cols,
            &primary_shell,
            cwd,
            env,
            self.last_output.clone(),
            self.output_notify.clone(),
            self.pending_graphics.clone(),
            self.current_cwd.clone(),
        );
        if spawned.is_none() && primary_shell != fallback_shell {
            tracing::info!("Retrying PTY spawn with fallback shell: {}", fallback_shell);
            spawned = Self::spawn_with_shell(
                rows,
                cols,
                &fallback_shell,
                cwd,
                env,
                self.last_output.clone(),
                self.output_notify.clone(),
                self.pending_graphics.clone(),
                self.current_cwd.clone(),
            );
        }

        if let Some(state) = spawned {
            *self.last_output.lock().unwrap() = Instant::now();
            self.pty = Some(state);
        }
    }
}

impl Default for PtyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalProvider for PtyProvider {
    fn spawn(&mut self, rows: u16, cols: u16) {
        self.current_size = (rows.max(1), cols.max(1)); // Track size
        self.ensure_running();
    }

    fn resize(&mut self, rows: u16, cols: u16) {
        // Skip if size hasn't changed
        if self.current_size == (rows, cols) {
            tracing::debug!("Resize skipped: size unchanged ({}, {})", rows, cols);
            return;
        }

        tracing::info!(
            "Resizing PTY from {:?} to ({}, {})",
            self.current_size,
            rows,
            cols
        );
        self.current_size = (rows, cols);

        if let Some(p) = &mut self.pty {
            let _ = p.master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
            p.emulator.lock().unwrap().resize(rows, cols);
        }
    }

    fn write(&mut self, bytes: &[u8]) {
        self.ensure_running();
        if let Some(p) = &mut self.pty {
            let _ = p.writer.write_all(bytes);
            let _ = p.writer.flush();
        }
    }

    fn paste(&mut self, text: &str) {
        self.ensure_running();
        let Some(p) = &mut self.pty else { return };
        let bracketed = p.emulator.lock().unwrap().bracketed_paste_enabled();
        if bracketed {
            let mut payload = Vec::with_capacity(text.len() + 16);
            payload.extend_from_slice(b"\x1b[200~");
            payload.extend_from_slice(text.as_bytes());
            payload.extend_from_slice(b"\x1b[201~");
            let _ = p.writer.write_all(&payload);
        } else {
            let _ = p.writer.write_all(text.as_bytes());
        }
        let _ = p.writer.flush();
    }

    fn mouse_mode_enabled(&self) -> bool {
        let Some(pty) = &self.pty else { return false };
        pty.emulator.lock().unwrap().mouse_mode_enabled()
    }

    fn bracketed_paste_enabled(&self) -> bool {
        let Some(pty) = &self.pty else { return false };
        pty.emulator.lock().unwrap().bracketed_paste_enabled()
    }

    fn get_screen(&self, rows: u16, cols: u16) -> ScreenContent {
        let Some(pty) = &self.pty else {
            tracing::debug!("get_screen: PTY not spawned");
            return ScreenContent::default();
        };
        let mut content = pty.emulator.lock().unwrap().get_screen(rows, cols);
        // Attach current working directory from OSC 7
        if let Ok(cwd) = self.current_cwd.lock() {
            content.cwd = cwd.clone();
        }
        tracing::debug!("get_screen: {} lines", content.lines.len());
        content
    }

    fn scroll(&mut self, delta: i32) {
        self.ensure_running();
        if let Some(pty) = &self.pty {
            pty.emulator.lock().unwrap().scroll(delta);
        }
    }

    fn take_pending_graphics(&self) -> Vec<Vec<u8>> {
        if let Ok(mut g) = self.pending_graphics.lock() {
            std::mem::take(&mut *g)
        } else {
            Vec::new()
        }
    }

    fn get_cwd(&self) -> Option<String> {
        // Primary: read /proc/<pid>/cwd (Linux) — works regardless of shell config
        #[cfg(target_os = "linux")]
        if let Some(pid) = self.child_pid() {
            let proc_cwd = std::path::PathBuf::from(format!("/proc/{}/cwd", pid));
            if let Ok(path) = std::fs::read_link(&proc_cwd) {
                return path.to_str().map(|s| s.to_string());
            }
        }

        // macOS fallback: lsof -p <pid> -Fn to find cwd (fd type 'cwd')
        #[cfg(target_os = "macos")]
        if let Some(pid) = self.child_pid() {
            if let Ok(out) = std::process::Command::new("lsof")
                .args(["-p", &pid.to_string(), "-Fn", "-a", "-d", "cwd"])
                .output()
            {
                // lsof -Fn prints "p<pid>\nn<path>" — grab the line starting with 'n'
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    if let Some(path) = line.strip_prefix('n') {
                        if path.starts_with('/') {
                            return Some(path.to_string());
                        }
                    }
                }
            }
        }

        // Last resort: OSC 7 path reported by shell
        self.current_cwd.lock().ok().and_then(|g| g.clone())
    }
}
