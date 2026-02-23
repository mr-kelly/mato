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

pub const BUILTIN_THEMES: &[&str] = &[
    "system",
    "tomato",
    "potato",
    "one-dark",
    "nord",
    "darcula",
    "solarized",
    "starship",
    "p10k",
    "gruvbox",
    "catppuccin",
    "navy",
];

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
        "tomato" => ThemeColors {
            follow_terminal: false,
            bg: [20, 15, 15],           // Warm dark black
            surface: [35, 25, 25],      // Warm dark surface
            border: [80, 45, 45],       // Muted tomato-skin border
            accent: [230, 57, 70],      // High-end Tomato Red (#E63946)
            accent2: [80, 220, 160],    // Leaf Green
            fg: [241, 250, 238],        // Off-white for high contrast
            fg_dim: [160, 140, 140],    // Warm dimmed text
            sel_bg: [100, 35, 35],      // Deep focused red
        },
        "potato" => ThemeColors {
            follow_terminal: false,
            bg: [40, 35, 30],           // Earthy dark brown
            surface: [55, 50, 45],      // Muted potato-skin
            border: [100, 90, 80],      // Earthy border
            accent: [212, 163, 115],    // Potato gold
            accent2: [233, 196, 106],   // Flashy yellow
            fg: [241, 234, 218],        // Warm parchment
            fg_dim: [140, 130, 120],    // Muted earthy text
            sel_bg: [85, 75, 65],       // Deep earthy selection
        },
        "one-dark" => ThemeColors {
            follow_terminal: false,
            bg: [40, 44, 52],
            surface: [44, 50, 60],
            border: [70, 75, 90],
            accent: [97, 175, 239],     // One Dark Blue
            accent2: [152, 195, 121],   // One Dark Green
            fg: [171, 178, 191],
            fg_dim: [92, 99, 112],
            sel_bg: [62, 68, 81],
        },
        "nord" => ThemeColors {
            follow_terminal: false,
            bg: [46, 52, 64],
            surface: [59, 66, 82],
            border: [76, 86, 106],
            accent: [136, 192, 208],    // Frost blue
            accent2: [163, 190, 140],   // Frost green
            fg: [216, 222, 233],
            fg_dim: [103, 110, 125],
            sel_bg: [67, 76, 94],
        },
        "darcula" => ThemeColors {
            follow_terminal: false,
            bg: [43, 43, 43],
            surface: [60, 63, 65],
            border: [85, 85, 85],
            accent: [187, 134, 252],    // Purple
            accent2: [106, 135, 89],    // Olive Green
            fg: [169, 183, 198],
            fg_dim: [128, 128, 128],
            sel_bg: [33, 66, 131],
        },
        "solarized" => ThemeColors {
            follow_terminal: false,
            bg: [0, 43, 54],
            surface: [7, 54, 66],
            border: [88, 110, 117],
            accent: [38, 139, 210],     // Solarized Blue
            accent2: [133, 153, 0],     // Solarized Green
            fg: [131, 148, 150],
            fg_dim: [101, 123, 131],
            sel_bg: [0, 33, 43],
        },
        "starship" => ThemeColors {
            follow_terminal: false,
            bg: [28, 28, 28],
            surface: [36, 36, 36],
            border: [64, 64, 64],
            accent: [255, 0, 127],      // Neon Magenta
            accent2: [0, 255, 255],     // Neon Cyan
            fg: [255, 255, 255],
            fg_dim: [160, 160, 160],
            sel_bg: [48, 48, 48],
        },
        "p10k" => ThemeColors {
            follow_terminal: false,
            bg: [10, 10, 10],           // Near black
            surface: [30, 30, 30],
            border: [50, 50, 50],
            accent: [0, 255, 0],        // Matrix Green
            accent2: [255, 255, 0],     // Laser Yellow
            fg: [255, 255, 255],
            fg_dim: [150, 150, 150],
            sel_bg: [60, 60, 60],
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
            // system (default/fallback)
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
