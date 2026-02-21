use std::{io::{Read, Write}, sync::{Arc, Mutex}, thread};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use ratatui::style::Color;
use crate::terminal_provider::{TerminalProvider, ScreenContent, ScreenLine, ScreenCell};

pub struct PtyProvider {
    pty: Option<PtyState>,
}

struct PtyState {
    writer: Box<dyn Write + Send>,
    parser: Arc<Mutex<vt100::Parser>>,
    master: Box<dyn portable_pty::MasterPty + Send>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl PtyProvider {
    pub fn new() -> Self {
        Self { pty: None }
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
        let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 0)));
        let p2 = Arc::clone(&parser);
        let mut reader = pair.master.try_clone_reader().expect("reader");
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop { match reader.read(&mut buf) { Ok(0) | Err(_) => break, Ok(n) => p2.lock().unwrap().process(&buf[..n]) } }
        });
        self.pty = Some(PtyState {
            writer: pair.master.take_writer().expect("writer"),
            parser,
            master: pair.master,
            _child: child,
        });
    }

    fn resize(&mut self, rows: u16, cols: u16) {
        if let Some(p) = &mut self.pty {
            let _ = p.master.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 });
            p.parser.lock().unwrap().set_size(rows, cols);
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
        let parser = pty.parser.lock().unwrap();
        let screen = parser.screen();
        
        let mut lines = vec![];
        for row in 0..rows {
            let mut cells = vec![];
            for col in 0..cols {
                let def = vt100::Cell::default();
                let cell = screen.cell(row, col).unwrap_or(&def);
                let ch = cell.contents().chars().next().unwrap_or(' ');
                cells.push(ScreenCell {
                    ch,
                    fg: vt_color(cell.fgcolor()),
                    bg: vt_color(cell.bgcolor()),
                    bold: cell.bold(),
                    italic: cell.italic(),
                    underline: cell.underline(),
                });
            }
            lines.push(ScreenLine { cells });
        }
        
        let (cr, cc) = screen.cursor_position();
        ScreenContent { lines, cursor: (cr, cc) }
    }
}

fn vt_color(c: vt100::Color) -> Option<Color> {
    match c {
        vt100::Color::Rgb(r, g, b) => Some(Color::Rgb(r, g, b)),
        vt100::Color::Idx(i) => Some(Color::Indexed(i)),
        vt100::Color::Default => None,
    }
}
