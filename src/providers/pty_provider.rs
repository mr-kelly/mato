use crate::emulators::{AlacrittyEmulator, Vt100Emulator};
use crate::terminal_emulator::TerminalEmulator;
use crate::terminal_provider::{ScreenContent, TerminalProvider};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::{
    env,
    io::{Read, Write},
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

pub struct PtyProvider {
    pty: Option<PtyState>,
    pub last_output: Arc<Mutex<Instant>>,
    current_size: (u16, u16), // Track current size to avoid unnecessary resizes
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
            current_size: (24, 80), // Default size
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

    fn spawn_with_shell(
        rows: u16,
        cols: u16,
        shell: &str,
        last_output: Arc<Mutex<Instant>>,
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
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        emulator_clone.lock().unwrap().process(&buf[..n]);
                        *last_output_clone.lock().unwrap() = Instant::now();
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

        let primary_shell = Self::resolve_shell();
        let fallback_shell = "/bin/sh".to_string();
        let mut spawned = Self::spawn_with_shell(rows, cols, &primary_shell, self.last_output.clone());
        if spawned.is_none() && primary_shell != fallback_shell {
            tracing::info!("Retrying PTY spawn with fallback shell: {}", fallback_shell);
            spawned = Self::spawn_with_shell(rows, cols, &fallback_shell, self.last_output.clone());
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
        let content = pty.emulator.lock().unwrap().get_screen(rows, cols);
        tracing::debug!("get_screen: {} lines", content.lines.len());
        content
    }

    fn scroll(&mut self, delta: i32) {
        self.ensure_running();
        if let Some(pty) = &self.pty {
            pty.emulator.lock().unwrap().scroll(delta);
        }
    }
}
