use std::sync::{Arc, Mutex};
use alacritty_terminal::Term;
use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::grid::Scroll;
use alacritty_terminal::term::Config;
use alacritty_terminal::vte::ansi::{Processor, CursorShape as AlacrittyCursorShape};
use alacritty_terminal::index::{Column, Line};
use ratatui::style::Color;

use crate::terminal_emulator::TerminalEmulator;
use crate::terminal_provider::{ScreenContent, ScreenLine, ScreenCell, CursorShape};

#[derive(Clone)]
struct TitleCapture(Arc<Mutex<Option<String>>>);

impl EventListener for TitleCapture {
    fn send_event(&self, event: Event) {
        if let Event::Title(t) = event {
            *self.0.lock().unwrap() = Some(t);
        }
    }
}

struct TermSize {
    cols: usize,
    lines: usize,
}

impl Dimensions for TermSize {
    fn columns(&self) -> usize { self.cols }
    fn screen_lines(&self) -> usize { self.lines }
    fn total_lines(&self) -> usize { self.lines }
}

pub struct AlacrittyEmulator {
    term: Term<TitleCapture>,
    processor: Processor,
    title: Arc<Mutex<Option<String>>>,
    scroll_offset: i32,
}

impl AlacrittyEmulator {
    pub fn new(rows: u16, cols: u16) -> Self {
        let title = Arc::new(Mutex::new(None));
        let listener = TitleCapture(Arc::clone(&title));
        let size = TermSize { cols: cols as usize, lines: rows as usize };
        let mut config = Config::default();
        config.scrolling_history = 10000;
        let term = Term::new(config, &size, listener);
        Self { term, processor: Processor::new(), title, scroll_offset: 0 }
    }
}

impl TerminalEmulator for AlacrittyEmulator {
    fn process(&mut self, bytes: &[u8]) {
        self.processor.advance(&mut self.term, bytes);
        // Auto-scroll to bottom on new output
        self.scroll_offset = 0;
    }

    fn scroll(&mut self, delta: i32) {
        self.scroll_offset += delta;
        let max = self.term.grid().history_size() as i32;
        self.scroll_offset = self.scroll_offset.clamp(0, max);
        if self.scroll_offset == 0 {
            self.term.scroll_display(Scroll::Bottom);
        } else {
            self.term.scroll_display(Scroll::Delta(-delta));
        }
    }

    fn get_screen(&self, rows: u16, cols: u16) -> ScreenContent {
        let grid = self.term.grid();
        let cursor = grid.cursor.point;
        let cursor_style = self.term.cursor_style();
        let title = self.title.lock().unwrap().clone();

        let cursor_shape = match cursor_style.shape {
            AlacrittyCursorShape::Beam => CursorShape::Beam,
            AlacrittyCursorShape::Underline => CursorShape::Underline,
            _ => CursorShape::Block,
        };

        let mut lines = Vec::with_capacity(rows as usize);
        let grid_rows = self.term.screen_lines() as i32;
        let grid_cols = self.term.columns();
        let render_rows = (rows as i32).min(grid_rows);
        let render_cols = cols.min(grid_cols as u16);

        for line in 0..render_rows {
            let mut cells = Vec::with_capacity(cols as usize);
            for col in 0..render_cols as usize {
                let cell = &grid[Line(line)][Column(col)];
                cells.push(ScreenCell {
                    ch: cell.c,
                    fg: ansi_color_to_ratatui(cell.fg),
                    bg: ansi_color_to_ratatui(cell.bg),
                    bold: cell.flags.contains(alacritty_terminal::term::cell::Flags::BOLD),
                    italic: cell.flags.contains(alacritty_terminal::term::cell::Flags::ITALIC),
                    underline: cell.flags.contains(alacritty_terminal::term::cell::Flags::UNDERLINE),
                });
            }
            lines.push(ScreenLine { cells });
        }

        ScreenContent { lines, cursor: (cursor.line.0 as u16, cursor.column.0 as u16), title, cursor_shape }
    }

    fn resize(&mut self, _rows: u16, _cols: u16) {
        // Do NOT resize the terminal emulator - that clears the screen.
        // The PTY process size (TIOCSWINSZ) is handled by PtyProvider directly.
        // We render by clipping/padding in get_screen.
    }
}

fn ansi_color_to_ratatui(color: alacritty_terminal::vte::ansi::Color) -> Option<Color> {
    use alacritty_terminal::vte::ansi::Color as AC;
    use alacritty_terminal::vte::ansi::NamedColor;
    match color {
        AC::Named(NamedColor::Background) | AC::Named(NamedColor::Foreground) => None,
        AC::Named(n) => Some(named_to_ratatui(n)),
        AC::Spec(rgb) => Some(Color::Rgb(rgb.r, rgb.g, rgb.b)),
        AC::Indexed(i) => Some(Color::Indexed(i)),
    }
}

fn named_to_ratatui(n: alacritty_terminal::vte::ansi::NamedColor) -> Color {
    use alacritty_terminal::vte::ansi::NamedColor::*;
    match n {
        Black | DimBlack => Color::Black,
        Red | DimRed => Color::Red,
        Green | DimGreen => Color::Green,
        Yellow | DimYellow => Color::Yellow,
        Blue | DimBlue => Color::Blue,
        Magenta | DimMagenta => Color::Magenta,
        Cyan | DimCyan => Color::Cyan,
        White | DimWhite => Color::White,
        BrightBlack => Color::DarkGray,
        BrightRed => Color::LightRed,
        BrightGreen => Color::LightGreen,
        BrightYellow => Color::LightYellow,
        BrightBlue => Color::LightBlue,
        BrightMagenta => Color::LightMagenta,
        BrightCyan => Color::LightCyan,
        BrightWhite => Color::Gray,
        _ => Color::Reset,
    }
}
