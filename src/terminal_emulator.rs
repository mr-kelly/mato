use crate::terminal_provider::ScreenContent;

/// Terminal emulator abstraction
/// Allows pluggable backends: vt100, alacritty_terminal, vte, etc.
pub trait TerminalEmulator: Send {
    fn process(&mut self, bytes: &[u8]);
    fn get_screen(&self, rows: u16, cols: u16) -> ScreenContent;
    fn resize(&mut self, rows: u16, cols: u16);
    fn scroll(&mut self, _delta: i32) {} // positive = scroll up (into history)
    fn bracketed_paste_enabled(&self) -> bool { false }
    fn mouse_mode_enabled(&self) -> bool { false }
}
