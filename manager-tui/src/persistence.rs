use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::app::App;

#[derive(Serialize, Deserialize, Clone)]
pub struct SavedTab {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct SavedTask {
    pub id: String,
    pub name: String,
    pub tabs: Vec<SavedTab>,
    pub active_tab: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SavedState {
    pub tasks: Vec<SavedTask>,
}

pub fn save_state(app: &App) {
    let state = SavedState {
        tasks: app.tasks.iter().map(|t| SavedTask {
            id: t.id.clone(),
            name: t.name.clone(),
            tabs: t.tabs.iter().map(|tb| SavedTab { id: tb.id.clone(), name: tb.name.clone() }).collect(),
            active_tab: t.active_tab,
        }).collect(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&state) {
        std::fs::write(state_path(), json).ok();
    }
}

pub fn load_state() -> Option<SavedState> {
    let json = std::fs::read_to_string(state_path()).ok()?;
    serde_json::from_str(&json).ok()
}

fn state_path() -> PathBuf {
    let mut p = config_dir();
    p.push("sandagent-tui");
    std::fs::create_dir_all(&p).ok();
    p.push("state.json");
    p
}

fn config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|_| {
        let mut h = PathBuf::from(std::env::var("HOME").unwrap_or_default());
        h.push(".config");
        h
    })
}
