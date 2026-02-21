use crate::terminal_emulator::TerminalEmulator;
use crate::terminal_provider::{ScreenContent, ScreenLine, ScreenCell};
use ratatui::style::Color;
use std::sync::{Arc, Mutex};

pub struct Vt100Emulator {
    parser: Arc<Mutex<vt100::Parser>>,
}

impl Vt100Emulator {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            parser: Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 0))),
        }
    }
}

impl TerminalEmulator for Vt100Emulator {
    fn process(&mut self, bytes: &[u8]) {
        self.parser.lock().unwrap().process(bytes);
    }
    
    fn get_screen(&self, rows: u16, cols: u16) -> ScreenContent {
        let parser = self.parser.lock().unwrap();
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
        let cr = cr.min(rows.saturating_sub(1));
        let cc = cc.min(cols.saturating_sub(1));
        ScreenContent { lines, cursor: (cr, cc), title: None, cursor_shape: crate::terminal_provider::CursorShape::Block }
    }
    
    fn resize(&mut self, rows: u16, cols: u16) {
        let mut parser = self.parser.lock().unwrap();
        let screen = parser.screen();
        let current_size = screen.size();
        
        // Only resize if size actually changed
        if current_size != (rows, cols) {
            tracing::warn!("Resizing vt100 from {:?} to ({}, {}) - content will be lost (vt100 limitation)", current_size, rows, cols);
            
            // vt100 Parser doesn't support content-preserving resize
            // We have to create a new parser, which clears the screen
            // This is a known limitation of the vt100 crate
            //
            // Workaround: Don't resize unless absolutely necessary
            // The PTY will continue to work, just with wrong size reported
            
            // For now, we still resize to avoid terminal corruption
            *parser = vt100::Parser::new(rows, cols, 0);
        }
    }
}

fn vt_color(c: vt100::Color) -> Option<Color> {
    match c {
        vt100::Color::Rgb(r, g, b) => Some(Color::Rgb(r, g, b)),
        vt100::Color::Idx(i) => Some(Color::Indexed(i)),
        vt100::Color::Default => None,
    }
}
