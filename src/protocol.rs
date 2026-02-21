use serde::{Deserialize, Serialize};
use crate::terminal_provider::ScreenContent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMsg {
    Hello { version: String },
    Spawn { tab_id: String, rows: u16, cols: u16 },
    Input { tab_id: String, data: Vec<u8> },
    Resize { tab_id: String, rows: u16, cols: u16 },
    GetScreen { tab_id: String, rows: u16, cols: u16 },
    GetIdleStatus,
    GetUpdateStatus,
    ClosePty { tab_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
    Welcome { version: String },
    Screen { tab_id: String, content: ScreenContent },
    Error { message: String },
    /// tab_id -> idle seconds
    IdleStatus { tabs: Vec<(String, u64)> },
    /// None = up to date or check failed; Some(ver) = update available
    UpdateStatus { latest: Option<String> },
}
