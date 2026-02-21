use crate::terminal_emulator::TerminalEmulator;
use crate::terminal_provider::{ScreenContent, ScreenLine, ScreenCell};
use ratatui::style::Color;
use std::sync::{Arc, Mutex};
use vte::{Parser, Perform};

/// VTE-based terminal emulator (used by GNOME Terminal, Alacritty)
/// Better ANSI compatibility than vt100
pub struct VteEmulator {
    state: Arc<Mutex<TerminalState>>,
    parser: Parser,
}

struct TerminalState {
    rows: u16,
    cols: u16,
    grid: Vec<Vec<Cell>>,
    cursor: (u16, u16),
}

#[derive(Clone)]
struct Cell {
    ch: char,
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
    italic: bool,
    underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

impl TerminalState {
    fn new(rows: u16, cols: u16) -> Self {
        let grid = vec![vec![Cell::default(); cols as usize]; rows as usize];
        Self {
            rows,
            cols,
            grid,
            cursor: (0, 0),
        }
    }
    
    fn put_char(&mut self, ch: char) {
        let (row, col) = self.cursor;
        if row < self.rows && col < self.cols {
            self.grid[row as usize][col as usize].ch = ch;
            // Move cursor right
            if col + 1 < self.cols {
                self.cursor.1 += 1;
            }
        }
    }
}

impl Perform for TerminalState {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }
    
    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                // Line feed - move down
                if self.cursor.0 + 1 < self.rows {
                    self.cursor.0 += 1;
                }
            }
            b'\r' => {
                // Carriage return - move to start of line
                self.cursor.1 = 0;
            }
            b'\x08' => {
                // Backspace
                if self.cursor.1 > 0 {
                    self.cursor.1 -= 1;
                }
            }
            _ => {}
        }
    }
    
    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _c: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
    fn csi_dispatch(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _c: char) {
        // TODO: Implement CSI sequences for cursor movement, colors, etc.
    }
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

impl VteEmulator {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            state: Arc::new(Mutex::new(TerminalState::new(rows, cols))),
            parser: Parser::new(),
        }
    }
}

impl TerminalEmulator for VteEmulator {
    fn process(&mut self, bytes: &[u8]) {
        let mut state = self.state.lock().unwrap();
        for &byte in bytes {
            self.parser.advance(&mut *state, &[byte]);
        }
    }
    
    fn get_screen(&self, rows: u16, cols: u16) -> ScreenContent {
        let state = self.state.lock().unwrap();
        let mut lines = vec![];
        
        for row in 0..rows.min(state.rows) {
            let mut cells = vec![];
            for col in 0..cols.min(state.cols) {
                let cell = &state.grid[row as usize][col as usize];
                cells.push(ScreenCell {
                    ch: cell.ch,
                    fg: cell.fg,
                    bg: cell.bg,
                    bold: cell.bold,
                    italic: cell.italic,
                    underline: cell.underline,
                });
            }
            lines.push(ScreenLine { cells });
        }
        
        ScreenContent {
            lines,
            cursor: state.cursor,
        }
    }
    
    fn resize(&mut self, rows: u16, cols: u16) {
        let mut state = self.state.lock().unwrap();
        // Only recreate if size actually changed
        if state.rows != rows || state.cols != cols {
            *state = TerminalState::new(rows, cols);
        }
    }
}
