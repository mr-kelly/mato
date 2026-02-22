use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Terminal backend provider trait
pub trait TerminalProvider: Send {
    fn spawn(&mut self, rows: u16, cols: u16);
    fn resize(&mut self, rows: u16, cols: u16);
    fn write(&mut self, bytes: &[u8]);
    fn paste(&mut self, text: &str) {
        self.write(text.as_bytes());
    }
    fn mouse_mode_enabled(&self) -> bool {
        false
    }
    fn bracketed_paste_enabled(&self) -> bool {
        false
    }
    fn get_screen(&self, rows: u16, cols: u16) -> ScreenContent;
    fn scroll(&mut self, _delta: i32) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenContent {
    pub lines: Vec<ScreenLine>,
    pub cursor: (u16, u16),
    pub title: Option<String>,
    pub cursor_shape: CursorShape,
    /// Bell (BEL) was triggered since last screen fetch.
    #[serde(default)]
    pub bell: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CursorShape {
    Block,
    Beam,
    Underline,
    Hidden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenLine {
    pub cells: Vec<ScreenCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCell {
    pub ch: char,
    #[serde(default = "default_display_width")]
    pub display_width: u8,
    #[serde(with = "color_serde")]
    pub fg: Option<Color>,
    #[serde(with = "color_serde")]
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    #[serde(default)]
    pub dim: bool,
    #[serde(default)]
    pub reverse: bool,
    #[serde(default)]
    pub strikethrough: bool,
    #[serde(default)]
    pub hidden: bool,
    /// Underline color (SGR 58); None means use fg color.
    #[serde(default, with = "color_serde")]
    pub underline_color: Option<Color>,
    /// Zero-width / combining characters attached to this cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zerowidth: Option<Vec<char>>,
}

fn default_display_width() -> u8 {
    1
}

mod color_serde {
    use ratatui::style::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(color: &Option<Color>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match color {
            Some(Color::Rgb(r, g, b)) => (Some("rgb"), Some([*r, *g, *b])).serialize(s),
            Some(Color::Indexed(i)) => (Some("idx"), Some([*i, 0, 0])).serialize(s),
            _ => (None::<&str>, None::<[u8; 3]>).serialize(s),
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Option<Color>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (ty, val): (Option<String>, Option<[u8; 3]>) = Deserialize::deserialize(d)?;
        Ok(match (ty.as_deref(), val) {
            (Some("rgb"), Some([r, g, b])) => Some(Color::Rgb(r, g, b)),
            (Some("idx"), Some([i, _, _])) => Some(Color::Indexed(i)),
            _ => None,
        })
    }
}

impl Default for ScreenContent {
    fn default() -> Self {
        Self {
            lines: vec![],
            cursor: (0, 0),
            title: None,
            cursor_shape: CursorShape::Block,
            bell: false,
        }
    }
}
