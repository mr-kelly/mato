use crate::terminal_provider::{CursorShape, ScreenContent, ScreenLine};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMsg {
    Hello {
        version: String,
    },
    Spawn {
        tab_id: String,
        rows: u16,
        cols: u16,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shell: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        env: Option<Vec<(String, String)>>,
    },
    Input {
        tab_id: String,
        data: Vec<u8>,
    },
    Paste {
        tab_id: String,
        data: String,
    },
    GetInputModes {
        tab_id: String,
    },
    Resize {
        tab_id: String,
        rows: u16,
        cols: u16,
    },
    GetScreen {
        tab_id: String,
        rows: u16,
        cols: u16,
    },
    GetIdleStatus,
    GetProcessStatus,
    GetUpdateStatus,
    ClosePty {
        tab_id: String,
    },
    Scroll {
        tab_id: String,
        delta: i32,
    },
    /// Subscribe to push-based screen updates for a tab.
    /// Daemon will push Screen/ScreenUnchanged whenever PTY has new output.
    Subscribe {
        tab_id: String,
        rows: u16,
        cols: u16,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
    Welcome {
        version: String,
    },
    Screen {
        tab_id: String,
        content: ScreenContent,
    },
    /// Screen content hasn't changed since last GetScreen on this connection.
    ScreenUnchanged,
    /// Incremental screen update: only changed lines + cursor/metadata.
    ScreenDiff {
        changed_lines: Vec<(u16, ScreenLine)>,
        cursor: (u16, u16),
        cursor_shape: CursorShape,
        title: Option<String>,
        #[serde(default)]
        bell: bool,
        #[serde(default)]
        focus_events_enabled: bool,
    },
    Error {
        message: String,
    },
    /// tab_id -> idle seconds
    IdleStatus {
        tabs: Vec<(String, u64)>,
    },
    /// tab_id -> child pid for each running PTY-backed tab
    ProcessStatus {
        tabs: Vec<(String, u32)>,
    },
    /// None = up to date or check failed; Some(ver) = update available
    UpdateStatus {
        latest: Option<String>,
    },
    InputModes {
        mouse: bool,
        bracketed_paste: bool,
    },
    /// Graphics passthrough: Kitty graphics protocol / Sixel / iTerm2 inline image APC
    /// sequences intercepted from PTY output. The client should re-emit them to the
    /// outer terminal (kitty/ghostty/wezterm/iTerm2) at the translated screen position.
    ///
    /// `cursor` is the display cursor in content-area coordinates at the time the
    /// last APC was captured (row, col), 0-indexed within the content area.
    Graphics {
        tab_id: String,
        /// Display cursor position (row, col) relative to content area at APC capture time.
        cursor: (u16, u16),
        /// Each entry is one complete `\x1b_...\x1b\\` APC sequence.
        payloads: Vec<Vec<u8>>,
    },
}
