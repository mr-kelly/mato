use crate::terminal_provider::ScreenContent;

/// Terminal emulator abstraction
/// Allows pluggable backends: vt100, alacritty_terminal, vte, etc.
pub trait TerminalEmulator: Send {
    /// Process incoming bytes from PTY
    fn process(&mut self, bytes: &[u8]);
    
    /// Get current screen content
    fn get_screen(&self, rows: u16, cols: u16) -> ScreenContent;
    
    /// Resize the terminal
    fn resize(&mut self, rows: u16, cols: u16);
}
