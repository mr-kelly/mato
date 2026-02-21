use serde::{Deserialize, Serialize};
use crate::client::app::App;
use crate::error::{MatoError, Result};

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
    #[serde(default)]
    pub active_task: usize,
}

pub fn save_state(app: &App) -> Result<()> {
    let state = SavedState {
        tasks: app.tasks.iter().map(|t| SavedTask {
            id: t.id.clone(),
            name: t.name.clone(),
            tabs: t.tabs.iter().map(|tb| SavedTab { id: tb.id.clone(), name: tb.name.clone() }).collect(),
            active_tab: t.active_tab,
        }).collect(),
        active_task: app.active_task(),
    };
    
    let json = serde_json::to_string_pretty(&state)?;
    let path = crate::utils::get_state_file_path();
    
    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| MatoError::StateSaveFailed(format!("Cannot create directory: {}", e)))?;
    }
    
    std::fs::write(&path, json)
        .map_err(|e| MatoError::StateSaveFailed(format!("Cannot write to {}: {}", path.display(), e)))?;
    
    Ok(())
}

pub fn load_state() -> Result<SavedState> {
    let path = crate::utils::get_state_file_path();
    
    let json = std::fs::read_to_string(&path)
        .map_err(|e| MatoError::StateLoadFailed(format!("Cannot read {}: {}", path.display(), e)))?;
    
    serde_json::from_str(&json)
        .map_err(|e| MatoError::StateParseFailed(format!("Invalid JSON in {}: {}", path.display(), e)))
}
