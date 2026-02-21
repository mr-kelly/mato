use std::{io::{Read, Write}, sync::{Arc, Mutex}, thread, time::Instant};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use crate::terminal_provider::{TerminalProvider, ScreenContent};
use crate::terminal_emulator::TerminalEmulator;
use crate::emulators::{Vt100Emulator, VteEmulator};

pub struct PtyProvider {
    pty: Option<PtyState>,
    pub last_output: Arc<Mutex<Instant>>,
}

struct PtyState {
    writer: Box<dyn Write + Send>,
    emulator: Arc<Mutex<Box<dyn TerminalEmulator>>>,
    master: Box<dyn portable_pty::MasterPty + Send>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl PtyProvider {
    pub fn new() -> Self {
        Self { pty: None, last_output: Arc::new(Mutex::new(Instant::now())) }
    }
    
    fn create_emulator(rows: u16, cols: u16) -> Box<dyn TerminalEmulator> {
        // Load config
        let config = crate::config::Config::load();
        
        match config.emulator.as_str() {
            "vte" => {
                tracing::info!("Using VTE emulator (from config)");
                Box::new(VteEmulator::new(rows, cols))
            }
            "vt100" | _ => {
                tracing::info!("Using VT100 emulator (from config)");
                Box::new(Vt100Emulator::new(rows, cols))
            }
        }
    }
}

impl TerminalProvider for PtyProvider {
    fn spawn(&mut self, rows: u16, cols: u16) {
        if self.pty.is_some() { return; }
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }).expect("openpty");
        let mut cmd = CommandBuilder::new("bash");
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

    fn get_screen(&self, rows: u16, cols: u16) -> ScreenContent {
        let Some(pty) = &self.pty else { return ScreenContent::default(); };
        pty.emulator.lock().unwrap().get_screen(rows, cols)
    }
}
