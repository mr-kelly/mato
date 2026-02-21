use ratatui::style::Color;

/// Terminal backend provider trait
pub trait TerminalProvider: Send {
    fn spawn(&mut self, rows: u16, cols: u16);
    fn resize(&mut self, rows: u16, cols: u16);
    fn write(&mut self, bytes: &[u8]);
    fn get_screen(&self, rows: u16, cols: u16) -> ScreenContent;
}

pub struct ScreenContent {
    pub lines: Vec<ScreenLine>,
    pub cursor: (u16, u16),
}

pub struct ScreenLine {
    pub cells: Vec<ScreenCell>,
}

pub struct ScreenCell {
    pub ch: char,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for ScreenContent {
    fn default() -> Self {
        Self { lines: vec![], cursor: (0, 0) }
    }
}
