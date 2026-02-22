use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResizeStrategy {
    /// Resize PTY + emulator when window size changes (correct for TUI apps)
    Sync,
    /// Keep PTY at original size, clip/pad display (safe but TUI apps won't adapt)
    Fixed,
}

impl Default for ResizeStrategy {
    fn default() -> Self {
        Self::Sync
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_emulator")]
    pub emulator: String,
    #[serde(default)]
    pub resize_strategy: ResizeStrategy,
}

fn default_emulator() -> String {
    "alacritty".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            emulator: default_emulator(),
            resize_strategy: ResizeStrategy::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = crate::utils::get_config_file_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str(&content) {
                return config;
            }
        }
        Self::default()
    }
}
