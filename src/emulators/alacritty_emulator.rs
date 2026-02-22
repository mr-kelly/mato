use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::grid::Scroll;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{Config, TermMode};
use alacritty_terminal::vte::ansi::{CursorShape as AlacrittyCursorShape, Processor};
use alacritty_terminal::Term;
use ratatui::style::Color;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::terminal_emulator::TerminalEmulator;
use crate::terminal_provider::{CursorShape, ScreenCell, ScreenContent, ScreenLine};

#[derive(Clone)]
struct EventCapture {
    title: Arc<Mutex<Option<String>>>,
    bell: Arc<AtomicBool>,
}

impl EventListener for EventCapture {
    fn send_event(&self, event: Event) {
        match event {
            Event::Title(t) => *self.title.lock().unwrap() = Some(t),
            Event::ResetTitle => *self.title.lock().unwrap() = None,
            Event::Bell => self.bell.store(true, Ordering::Relaxed),
            _ => {}
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
    term: Term<EventCapture>,
    processor: Processor,
    title: Arc<Mutex<Option<String>>>,
    bell: Arc<AtomicBool>,
    scroll_offset: i32,
    theme_palette: Option<AnsiThemePalette>,
}

impl AlacrittyEmulator {
    pub fn new(rows: u16, cols: u16) -> Self {
        let title = Arc::new(Mutex::new(None));
        let bell = Arc::new(AtomicBool::new(false));
        let listener = EventCapture {
            title: Arc::clone(&title),
            bell: Arc::clone(&bell),
        };
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
            bell,
            scroll_offset: 0,
            theme_palette: if theme.follow_terminal {
                None
            } else {
                Some(palette_from_theme(&theme))
            },
        }
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
        let renderable = self.term.renderable_content();
        let mut cursor = renderable.cursor.point;
        let title = self.title.lock().unwrap().clone();

        // Use renderable cursor shape which accounts for DECTCEM (cursor visibility).
        // term.cursor_style() only gives the shape preference, not visibility.
        let cursor_shape = match renderable.cursor.shape {
            AlacrittyCursorShape::Beam => CursorShape::Beam,
            AlacrittyCursorShape::Underline => CursorShape::Underline,
            AlacrittyCursorShape::Hidden => CursorShape::Hidden,
            _ => CursorShape::Block,
        };

        // Some TUIs may report an off-screen renderable cursor transiently.
        // Only in that case fall back to raw grid cursor.
        let raw = grid.cursor.point;
        let renderable_offscreen = cursor.line.0 < 0 || cursor.line.0 >= rows as i32;
        if renderable_offscreen {
            cursor = raw;
        }
        if cursor.line.0 >= 0 && cursor.column.0 > 0 {
            let point_cell = &grid[cursor.line][cursor.column];
            if point_cell.flags.intersects(Flags::WIDE_CHAR_SPACER) {
                cursor.column -= 1;
            }
        }

        let grid_cols = self.term.columns();
        let render_rows = (rows as usize).min(self.term.screen_lines());
        let render_cols = (cols as usize).min(grid_cols);

        let mut lines = vec![
            ScreenLine {
                cells: Vec::with_capacity(render_cols)
            };
            render_rows
        ];

        let mut first_visible_line: Option<i32> = None;
        for indexed in renderable.display_iter {
            let abs_line = indexed.point.line.0;
            let col = indexed.point.column.0;
            let base = *first_visible_line.get_or_insert(abs_line);
            let row = abs_line - base;
            if row < 0 {
                continue;
            }
            let row = row as usize;
            if row >= render_rows || col >= render_cols {
                continue;
            }
            let cell = indexed.cell;
            let spacer = cell
                .flags
                .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER);
            let display_width = if spacer {
                0
            } else if cell.flags.contains(Flags::WIDE_CHAR) {
                2
            } else {
                1
            };
            let zerowidth = if spacer {
                None
            } else {
                cell.zerowidth()
                    .filter(|zw| !zw.is_empty())
                    .map(|zw| zw.to_vec())
            };
            let underline_color = cell
                .underline_color()
                .and_then(|c| self.ansi_color_to_ratatui(c));
            lines[row].cells.push(ScreenCell {
                ch: if spacer { '\0' } else { cell.c },
                display_width,
                fg: self.ansi_color_to_ratatui(cell.fg),
                bg: self.ansi_color_to_ratatui(cell.bg),
                bold: cell.flags.contains(Flags::BOLD),
                italic: cell.flags.contains(Flags::ITALIC),
                underline: cell.flags.intersects(
                    Flags::UNDERLINE
                        | Flags::DOUBLE_UNDERLINE
                        | Flags::UNDERCURL
                        | Flags::DOTTED_UNDERLINE
                        | Flags::DASHED_UNDERLINE,
                ),
                dim: cell.flags.contains(Flags::DIM),
                reverse: cell.flags.contains(Flags::INVERSE),
                strikethrough: cell.flags.contains(Flags::STRIKEOUT),
                hidden: cell.flags.contains(Flags::HIDDEN),
                underline_color,
                zerowidth,
            });
        }

        // Ensure each row has enough cells for stable cursor/math paths.
        for line in &mut lines {
            while line.cells.len() < render_cols {
                line.cells.push(ScreenCell {
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
                });
            }
        }

        ScreenContent {
            lines,
            cursor: (cursor.line.0.max(0) as u16, cursor.column.0.max(0) as u16),
            title,
            cursor_shape,
            bell: self.bell.swap(false, Ordering::Relaxed),
        }
    }

    fn resize(&mut self, _rows: u16, _cols: u16) {
        // Do NOT resize the terminal emulator - that clears the screen.
        // The PTY process size (TIOCSWINSZ) is handled by PtyProvider directly.
        // We render by clipping/padding in get_screen.
    }

    fn bracketed_paste_enabled(&self) -> bool {
        self.term.mode().contains(TermMode::BRACKETED_PASTE)
    }

    fn mouse_mode_enabled(&self) -> bool {
        self.term.mode().intersects(TermMode::MOUSE_MODE)
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
