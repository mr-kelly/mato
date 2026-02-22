use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub follow_terminal: bool,
    pub bg: [u8; 3],
    pub surface: [u8; 3],
    pub border: [u8; 3],
    pub accent: [u8; 3],
    pub accent2: [u8; 3],
    pub fg: [u8; 3],
    pub fg_dim: [u8; 3],
    pub sel_bg: [u8; 3],
}

impl ThemeColors {
    pub fn bg(&self) -> Color {
        if self.follow_terminal {
            Color::Reset
        } else {
            Color::Rgb(self.bg[0], self.bg[1], self.bg[2])
        }
    }
    pub fn surface(&self) -> Color {
        if self.follow_terminal {
            Color::Reset
        } else {
            Color::Rgb(self.surface[0], self.surface[1], self.surface[2])
        }
    }
    pub fn border(&self) -> Color {
        if self.follow_terminal {
            Color::Reset
        } else {
            Color::Rgb(self.border[0], self.border[1], self.border[2])
        }
    }
    pub fn accent(&self) -> Color {
        if self.follow_terminal {
            Color::Reset
        } else {
            Color::Rgb(self.accent[0], self.accent[1], self.accent[2])
        }
    }
    pub fn accent2(&self) -> Color {
        if self.follow_terminal {
            Color::Reset
        } else {
            Color::Rgb(self.accent2[0], self.accent2[1], self.accent2[2])
        }
    }
    pub fn fg(&self) -> Color {
        if self.follow_terminal {
            Color::Reset
        } else {
            Color::Rgb(self.fg[0], self.fg[1], self.fg[2])
        }
    }
    pub fn fg_dim(&self) -> Color {
        if self.follow_terminal {
            Color::Reset
        } else {
            Color::Rgb(self.fg_dim[0], self.fg_dim[1], self.fg_dim[2])
        }
    }
    pub fn sel_bg(&self) -> Color {
        if self.follow_terminal {
            Color::Reset
        } else {
            Color::Rgb(self.sel_bg[0], self.sel_bg[1], self.sel_bg[2])
        }
    }
    pub fn rgb_bg(&self) -> [u8; 3] {
        self.bg
    }
    pub fn rgb_accent(&self) -> [u8; 3] {
        self.accent
    }
    pub fn rgb_accent2(&self) -> [u8; 3] {
        self.accent2
    }
    pub fn rgb_fg(&self) -> [u8; 3] {
        self.fg
    }
}

/// Top-level theme.toml structure.
/// `name` selects a built-in base; `colors` overrides individual values.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeFile {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub colors: PartialColors,
}

/// All fields optional — only specified ones override the base theme.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialColors {
    pub bg: Option<[u8; 3]>,
    pub surface: Option<[u8; 3]>,
    pub border: Option<[u8; 3]>,
    pub accent: Option<[u8; 3]>,
    pub accent2: Option<[u8; 3]>,
    pub fg: Option<[u8; 3]>,
    pub fg_dim: Option<[u8; 3]>,
    pub sel_bg: Option<[u8; 3]>,
}

pub const BUILTIN_THEMES: &[&str] = &["system", "navy", "gruvbox", "catppuccin"];

pub fn builtin(name: &str) -> ThemeColors {
    match name {
        "system" => ThemeColors {
            follow_terminal: true,
            bg: [0, 0, 0],
            surface: [0, 0, 0],
            border: [0, 0, 0],
            accent: [0, 0, 0],
            accent2: [0, 0, 0],
            fg: [0, 0, 0],
            fg_dim: [0, 0, 0],
            sel_bg: [0, 0, 0],
        },
        "gruvbox" => ThemeColors {
            follow_terminal: false,
            bg: [40, 40, 40],
            surface: [50, 48, 47],
            border: [102, 92, 84],
            accent: [250, 189, 47],
            accent2: [184, 187, 38],
            fg: [235, 219, 178],
            fg_dim: [146, 131, 116],
            sel_bg: [80, 73, 69],
        },
        "catppuccin" => ThemeColors {
            follow_terminal: false,
            bg: [30, 30, 46],
            surface: [49, 50, 68],
            border: [88, 91, 112],
            accent: [137, 180, 250],
            accent2: [166, 227, 161],
            fg: [205, 214, 244],
            fg_dim: [108, 112, 134],
            sel_bg: [69, 71, 90],
        },
        _ => ThemeColors {
            // navy (default)
            follow_terminal: false,
            bg: [18, 18, 28],
            surface: [28, 28, 42],
            border: [60, 60, 90],
            accent: [100, 160, 255],
            accent2: [80, 220, 160],
            fg: [210, 210, 230],
            fg_dim: [100, 100, 130],
            sel_bg: [40, 60, 100],
        },
    }
}

pub fn load() -> ThemeColors {
    let path = crate::utils::get_config_dir().join("theme.toml");
    let file: ThemeFile = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default();

    let base_name = file.name.as_deref().unwrap_or("system");
    let mut base = builtin(base_name);
    let o = &file.colors;
    if let Some(v) = o.bg {
        base.bg = v;
    }
    if let Some(v) = o.surface {
        base.surface = v;
    }
    if let Some(v) = o.border {
        base.border = v;
    }
    if let Some(v) = o.accent {
        base.accent = v;
    }
    if let Some(v) = o.accent2 {
        base.accent2 = v;
    }
    if let Some(v) = o.fg {
        base.fg = v;
    }
    if let Some(v) = o.fg_dim {
        base.fg_dim = v;
    }
    if let Some(v) = o.sel_bg {
        base.sel_bg = v;
    }
    base
}

pub fn save_name(name: &str) -> std::io::Result<()> {
    let dir = crate::utils::get_config_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("theme.toml");
    // Preserve existing color overrides if any
    let mut file: ThemeFile = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default();
    file.name = Some(name.to_string());
    let content = toml::to_string_pretty(&file).unwrap();
    std::fs::write(&path, content)
}

pub fn selected_name() -> String {
    let path = crate::utils::get_config_dir().join("theme.toml");
    let file: ThemeFile = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default();
    file.name.unwrap_or_else(|| "system".to_string())
}

pub fn selected_index() -> usize {
    let name = selected_name();
    BUILTIN_THEMES.iter().position(|n| *n == name).unwrap_or(0)
}
