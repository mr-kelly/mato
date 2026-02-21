use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_emulator")]
    pub emulator: String,
}

fn default_emulator() -> String {
    "vt100".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            emulator: default_emulator(),
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
    
    pub fn save(&self) -> std::io::Result<()> {
        let path = crate::utils::get_config_file_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).unwrap();
        std::fs::write(&path, content)
    }
}
