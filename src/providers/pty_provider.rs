use std::{env, io::{Read, Write}, sync::{Arc, Mutex}, thread, time::Instant};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use crate::terminal_provider::{TerminalProvider, ScreenContent};
use crate::terminal_emulator::TerminalEmulator;
use crate::emulators::{Vt100Emulator, AlacrittyEmulator};

pub struct PtyProvider {
    pty: Option<PtyState>,
    pub last_output: Arc<Mutex<Instant>>,
    current_size: (u16, u16), // Track current size to avoid unnecessary resizes
}

struct PtyState {
    writer: Box<dyn Write + Send>,
    emulator: Arc<Mutex<Box<dyn TerminalEmulator>>>,
    master: Box<dyn portable_pty::MasterPty + Send>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
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
        self.pty.as_ref().and_then(|p| p._child.process_id())
    }
}

impl Default for PtyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalProvider for PtyProvider {
    fn spawn(&mut self, rows: u16, cols: u16) {
        if self.pty.is_some() { return; }
        
        self.current_size = (rows, cols); // Track size
        
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }).expect("openpty");
        let shell = Self::resolve_shell();
        tracing::info!("Spawning PTY shell: {}", shell);
        let mut cmd = CommandBuilder::new(shell);
        cmd.env("TERM", "xterm-256color");
        let child = pair.slave.spawn_command(cmd).expect("spawn");
        
        // Use selected emulator
        let emulator = Self::create_emulator(rows, cols);
        let emulator = Arc::new(Mutex::new(emulator));
        let emulator_clone = Arc::clone(&emulator);
        let last_output = Arc::clone(&self.last_output);
        
        let mut reader = pair.master.try_clone_reader().expect("reader");
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop { 
                match reader.read(&mut buf) { 
                    Ok(0) | Err(_) => break, 
                    Ok(n) => {
                        emulator_clone.lock().unwrap().process(&buf[..n]);
                        *last_output.lock().unwrap() = Instant::now();
                    }
                } 
            }
        });
        
        self.pty = Some(PtyState {
            writer: pair.master.take_writer().expect("writer"),
            emulator,
            master: pair.master,
            _child: child,
        });
    }

    fn resize(&mut self, rows: u16, cols: u16) {
        // Skip if size hasn't changed
        if self.current_size == (rows, cols) {
            tracing::debug!("Resize skipped: size unchanged ({}, {})", rows, cols);
            return;
        }
        
        tracing::info!("Resizing PTY from {:?} to ({}, {})", self.current_size, rows, cols);
        self.current_size = (rows, cols);
        
        if let Some(p) = &mut self.pty {
            let _ = p.master.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 });
            p.emulator.lock().unwrap().resize(rows, cols);
        }
    }

    fn write(&mut self, bytes: &[u8]) {
        if let Some(p) = &mut self.pty {
            let _ = p.writer.write_all(bytes);
            let _ = p.writer.flush();
        }
    }

    fn paste(&mut self, text: &str) {
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
        if let Some(pty) = &self.pty {
            pty.emulator.lock().unwrap().scroll(delta);
        }
    }
}
