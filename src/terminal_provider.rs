/// Terminal provider trait - abstracts different backend implementations
pub trait TerminalProvider: Send {
    /// Spawn a new terminal session
    fn spawn(&mut self, rows: u16, cols: u16);
    
    /// Check if terminal is spawned
    fn is_spawned(&self) -> bool;
    
    /// Resize terminal
    fn resize(&mut self, rows: u16, cols: u16);
    
    /// Write bytes to terminal
    fn write(&mut self, bytes: &[u8]);
    
    /// Get screen content for rendering (returns lines of styled text)
    /// For PTY: vt100 parsed screen
    /// For tmux: captured pane content
    fn get_screen(&self, rows: u16, cols: u16) -> ScreenContent;
}

/// Screen content returned by provider
pub struct ScreenContent {
    pub lines: Vec<ScreenLine>,
    pub cursor: (u16, u16), // (row, col)
}

pub struct ScreenLine {
    pub cells: Vec<ScreenCell>,
}

pub struct ScreenCell {
    pub ch: char,
    pub fg: Option<(u8, u8, u8)>, // RGB or None for default
    pub bg: Option<(u8, u8, u8)>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for ScreenContent {
    fn default() -> Self {
        Self { lines: vec![], cursor: (0, 0) }
    }
}
