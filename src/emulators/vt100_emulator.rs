use crate::terminal_emulator::TerminalEmulator;
use crate::terminal_provider::{ScreenCell, ScreenContent, ScreenLine};
use ratatui::style::Color;
use std::sync::{Arc, Mutex};
use unicode_width::UnicodeWidthChar;

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
        let (screen_rows, screen_cols) = screen.size();

        let render_rows = rows.min(screen_rows);
        let render_cols = cols.min(screen_cols);
        // Show the BOTTOM render_rows rows (same as alacritty emulator).
        // When a smaller client subscribes to a PTY started by a larger client,
        // the shell/cursor lives at the bottom — we must show the bottom rows.
        let row_offset = screen_rows - render_rows;

        let mut lines = vec![];
        for display_row in 0..render_rows {
            let pty_row = row_offset + display_row;
            let mut cells = vec![];
            for col in 0..render_cols {
                let sc = if let Some(cell) = screen.cell(pty_row, col) {
                    let ch = cell.contents().chars().next().unwrap_or(' ');
                    let display_width = if ch == '\0' {
                        0
                    } else {
                        UnicodeWidthChar::width(ch).unwrap_or(1).clamp(1, 2) as u8
                    };
                    ScreenCell {
                        ch,
                        display_width,
                        fg: vt_color(cell.fgcolor()),
                        bg: vt_color(cell.bgcolor()),
                        bold: cell.bold(),
                        italic: cell.italic(),
                        underline: cell.underline(),
                        dim: false,
                        reverse: cell.inverse(),
                        strikethrough: false,
                        hidden: false,
                        underline_color: None,
                        zerowidth: None,
                    }
                } else {
                    ScreenCell {
                        ch: ' ',
                        display_width: 1,
                        fg: None,
                        bg: None,
                        bold: false,
                        italic: false,
                        underline: false,
                        dim: false,
                        reverse: false,
                        strikethrough: false,
                        hidden: false,
                        underline_color: None,
                        zerowidth: None,
                    }
                };
                cells.push(sc);
            }
            lines.push(ScreenLine { cells });
        }

        let (cr, cc) = screen.cursor_position();
        // Adjust cursor row to be relative to the display window (bottom render_rows rows).
        let display_cr = if cr >= row_offset {
            (cr - row_offset).min(render_rows.saturating_sub(1))
        } else {
            0
        };
        let display_cc = cc.min(render_cols.saturating_sub(1));
        ScreenContent {
            lines,
            cursor: (display_cr, display_cc),
            title: None,
            cursor_shape: crate::terminal_provider::CursorShape::Block,
            bell: false,
            focus_events_enabled: false,
        }
    }

    fn resize(&mut self, rows: u16, cols: u16) {
        let mut parser = self.parser.lock().unwrap();
        let screen = parser.screen();
        let current_size = screen.size();

        // Only resize if size actually changed
        if current_size != (rows, cols) {
            tracing::warn!(
                "Resizing vt100 from {:?} to ({}, {}) - content will be lost (vt100 limitation)",
                current_size,
                rows,
                cols
            );

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
