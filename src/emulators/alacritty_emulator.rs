use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::grid::Scroll;
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::Config;
use alacritty_terminal::vte::ansi::{CursorShape as AlacrittyCursorShape, Processor};
use alacritty_terminal::Term;
use ratatui::style::Color;
use std::sync::{Arc, Mutex};

use crate::terminal_emulator::TerminalEmulator;
use crate::terminal_provider::{CursorShape, ScreenCell, ScreenContent, ScreenLine};

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
    fn columns(&self) -> usize {
        self.cols
    }
    fn screen_lines(&self) -> usize {
        self.lines
    }
    fn total_lines(&self) -> usize {
        self.lines
    }
}

#[derive(Clone, Copy)]
struct AnsiThemePalette {
    normal: [Color; 8],
    bright: [Color; 8],
}

pub struct AlacrittyEmulator {
    term: Term<TitleCapture>,
    processor: Processor,
    title: Arc<Mutex<Option<String>>>,
    scroll_offset: i32,
    bracketed_paste: bool,
    mouse_mode: bool,
    mode_tail: Vec<u8>,
    theme_palette: Option<AnsiThemePalette>,
}

impl AlacrittyEmulator {
    pub fn new(rows: u16, cols: u16) -> Self {
        let title = Arc::new(Mutex::new(None));
        let listener = TitleCapture(Arc::clone(&title));
        let size = TermSize {
            cols: cols as usize,
            lines: rows as usize,
        };
        let config = Config {
            scrolling_history: 10000,
            ..Config::default()
        };
        let term = Term::new(config, &size, listener);
        let theme = crate::theme::load();
        Self {
            term,
            processor: Processor::new(),
            title,
            scroll_offset: 0,
            bracketed_paste: false,
            mouse_mode: false,
            mode_tail: Vec::new(),
            theme_palette: if theme.follow_terminal {
                None
            } else {
                Some(palette_from_theme(&theme))
            },
        }
    }

    fn update_terminal_modes(&mut self, bytes: &[u8]) {
        const ENABLE: &[u8] = b"\x1b[?2004h";
        const DISABLE: &[u8] = b"\x1b[?2004l";
        const MOUSE_ENABLE_1000: &[u8] = b"\x1b[?1000h";
        const MOUSE_ENABLE_1002: &[u8] = b"\x1b[?1002h";
        const MOUSE_ENABLE_1003: &[u8] = b"\x1b[?1003h";
        const MOUSE_DISABLE_1000: &[u8] = b"\x1b[?1000l";
        const MOUSE_DISABLE_1002: &[u8] = b"\x1b[?1002l";
        const MOUSE_DISABLE_1003: &[u8] = b"\x1b[?1003l";
        let mut merged = Vec::with_capacity(self.mode_tail.len() + bytes.len());
        merged.extend_from_slice(&self.mode_tail);
        merged.extend_from_slice(bytes);

        if merged.windows(ENABLE.len()).any(|w| w == ENABLE) {
            self.bracketed_paste = true;
        }
        if merged.windows(DISABLE.len()).any(|w| w == DISABLE) {
            self.bracketed_paste = false;
        }
        if merged
            .windows(MOUSE_ENABLE_1000.len())
            .any(|w| w == MOUSE_ENABLE_1000)
            || merged
                .windows(MOUSE_ENABLE_1002.len())
                .any(|w| w == MOUSE_ENABLE_1002)
            || merged
                .windows(MOUSE_ENABLE_1003.len())
                .any(|w| w == MOUSE_ENABLE_1003)
        {
            self.mouse_mode = true;
        }
        if merged
            .windows(MOUSE_DISABLE_1000.len())
            .any(|w| w == MOUSE_DISABLE_1000)
            || merged
                .windows(MOUSE_DISABLE_1002.len())
                .any(|w| w == MOUSE_DISABLE_1002)
            || merged
                .windows(MOUSE_DISABLE_1003.len())
                .any(|w| w == MOUSE_DISABLE_1003)
        {
            self.mouse_mode = false;
        }

        let keep = ENABLE.len().max(DISABLE.len()).saturating_sub(1);
        if merged.len() > keep {
            self.mode_tail = merged[merged.len() - keep..].to_vec();
        } else {
            self.mode_tail = merged;
        }
    }
}

impl TerminalEmulator for AlacrittyEmulator {
    fn process(&mut self, bytes: &[u8]) {
        self.processor.advance(&mut self.term, bytes);
        self.update_terminal_modes(bytes);
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
                    fg: self.ansi_color_to_ratatui(cell.fg),
                    bg: self.ansi_color_to_ratatui(cell.bg),
                    bold: cell
                        .flags
                        .contains(alacritty_terminal::term::cell::Flags::BOLD),
                    italic: cell
                        .flags
                        .contains(alacritty_terminal::term::cell::Flags::ITALIC),
                    underline: cell
                        .flags
                        .contains(alacritty_terminal::term::cell::Flags::UNDERLINE),
                });
            }
            lines.push(ScreenLine { cells });
        }

        ScreenContent {
            lines,
            cursor: (cursor.line.0 as u16, cursor.column.0 as u16),
            title,
            cursor_shape,
        }
    }

    fn resize(&mut self, _rows: u16, _cols: u16) {
        // Do NOT resize the terminal emulator - that clears the screen.
        // The PTY process size (TIOCSWINSZ) is handled by PtyProvider directly.
        // We render by clipping/padding in get_screen.
    }

    fn bracketed_paste_enabled(&self) -> bool {
        self.bracketed_paste
    }

    fn mouse_mode_enabled(&self) -> bool {
        self.mouse_mode
    }
}

impl AlacrittyEmulator {
    fn ansi_color_to_ratatui(&self, color: alacritty_terminal::vte::ansi::Color) -> Option<Color> {
        use alacritty_terminal::vte::ansi::Color as AC;
        use alacritty_terminal::vte::ansi::NamedColor;
        match color {
            AC::Named(NamedColor::Background) | AC::Named(NamedColor::Foreground) => None,
            AC::Named(n) => Some(self.named_to_ratatui(n)),
            AC::Spec(rgb) => Some(Color::Rgb(rgb.r, rgb.g, rgb.b)),
            AC::Indexed(i) => Some(Color::Indexed(i)),
        }
    }

    fn named_to_ratatui(&self, n: alacritty_terminal::vte::ansi::NamedColor) -> Color {
        use alacritty_terminal::vte::ansi::NamedColor::*;
        if let Some(p) = self.theme_palette {
            let idx = match n {
                Black | DimBlack => Some((false, 0)),
                Red | DimRed => Some((false, 1)),
                Green | DimGreen => Some((false, 2)),
                Yellow | DimYellow => Some((false, 3)),
                Blue | DimBlue => Some((false, 4)),
                Magenta | DimMagenta => Some((false, 5)),
                Cyan | DimCyan => Some((false, 6)),
                White | DimWhite => Some((false, 7)),
                BrightBlack => Some((true, 0)),
                BrightRed => Some((true, 1)),
                BrightGreen => Some((true, 2)),
                BrightYellow => Some((true, 3)),
                BrightBlue => Some((true, 4)),
                BrightMagenta => Some((true, 5)),
                BrightCyan => Some((true, 6)),
                BrightWhite => Some((true, 7)),
                _ => None,
            };
            if let Some((bright, i)) = idx {
                return if bright { p.bright[i] } else { p.normal[i] };
            }
        }
        named_to_ratatui_system(n)
    }
}

fn named_to_ratatui_system(n: alacritty_terminal::vte::ansi::NamedColor) -> Color {
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

fn palette_from_theme(theme: &crate::theme::ThemeColors) -> AnsiThemePalette {
    let bg = theme.rgb_bg();
    let fg = theme.rgb_fg();
    let accent = theme.rgb_accent();
    let accent2 = theme.rgb_accent2();
    let red = mix(accent, [255, 96, 96], 0.60);
    let green = mix(accent2, [96, 230, 130], 0.65);
    let yellow = mix(red, green, 0.5);
    let blue = accent;
    let magenta = mix(accent, [210, 120, 240], 0.55);
    let cyan = mix(accent2, [110, 220, 245], 0.55);
    let black = darken(bg, 0.45);
    let white = fg;

    let normal = [
        rgb(black),
        rgb(red),
        rgb(green),
        rgb(yellow),
        rgb(blue),
        rgb(magenta),
        rgb(cyan),
        rgb(white),
    ];
    let bright = [
        rgb(lighten(black, 0.35)),
        rgb(lighten(red, 0.25)),
        rgb(lighten(green, 0.20)),
        rgb(lighten(yellow, 0.18)),
        rgb(lighten(blue, 0.25)),
        rgb(lighten(magenta, 0.20)),
        rgb(lighten(cyan, 0.20)),
        rgb(lighten(white, 0.12)),
    ];
    AnsiThemePalette { normal, bright }
}

fn rgb(v: [u8; 3]) -> Color {
    Color::Rgb(v[0], v[1], v[2])
}

fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let clamped = t.clamp(0.0, 1.0);
    [
        ((a[0] as f32) * (1.0 - clamped) + (b[0] as f32) * clamped).round() as u8,
        ((a[1] as f32) * (1.0 - clamped) + (b[1] as f32) * clamped).round() as u8,
        ((a[2] as f32) * (1.0 - clamped) + (b[2] as f32) * clamped).round() as u8,
    ]
}

fn darken(c: [u8; 3], amt: f32) -> [u8; 3] {
    mix(c, [0, 0, 0], amt)
}

fn lighten(c: [u8; 3], amt: f32) -> [u8; 3] {
    mix(c, [255, 255, 255], amt)
}
