use crate::client::app::App;
use crate::error::{MatoError, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SavedTab {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct SavedDesk {
    pub id: String,
    pub name: String,
    pub tabs: Vec<SavedTab>,
    pub active_tab: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SavedOffice {
    pub id: String,
    pub name: String,
    pub desks: Vec<SavedDesk>,
    pub active_desk: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SavedState {
    pub offices: Vec<SavedOffice>,
    #[serde(default)]
    pub current_office: usize,
}

pub fn save_state(app: &App) -> Result<()> {
    let state = SavedState {
        offices: app
            .offices
            .iter()
            .map(|o| SavedOffice {
                id: o.id.clone(),
                name: o.name.clone(),
                desks: o
                    .desks
                    .iter()
                    .map(|d| SavedDesk {
                        id: d.id.clone(),
                        name: d.name.clone(),
                        tabs: d
                            .tabs
                            .iter()
                            .map(|tb| SavedTab {
                                id: tb.id.clone(),
                                name: tb.name.clone(),
                            })
                            .collect(),
                        active_tab: d.active_tab,
                    })
                    .collect(),
                active_desk: o.active_desk,
            })
            .collect(),
        current_office: app.current_office,
    };

    let json = serde_json::to_string_pretty(&state)?;
    let path = crate::utils::get_state_file_path();

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| MatoError::StateSaveFailed(format!("Cannot create directory: {}", e)))?;
    }

    std::fs::write(&path, json).map_err(|e| {
        MatoError::StateSaveFailed(format!("Cannot write to {}: {}", path.display(), e))
    })?;

    Ok(())
}

pub fn load_state() -> Result<SavedState> {
    let path = crate::utils::get_state_file_path();

    let json = std::fs::read_to_string(&path).map_err(|e| {
        MatoError::StateLoadFailed(format!("Cannot read {}: {}", path.display(), e))
    })?;

    serde_json::from_str(&json).map_err(|e| {
        MatoError::StateParseFailed(format!("Invalid JSON in {}: {}", path.display(), e))
    })
}
