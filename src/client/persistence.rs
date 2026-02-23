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

// Legacy format for backward compatibility
#[derive(Deserialize)]
struct LegacySavedOffice {
    desks: Vec<SavedDesk>,
    #[serde(default)]
    active_desk: usize,
}

#[derive(Deserialize)]
struct LegacySavedState {
    offices: Vec<LegacySavedOffice>,
    #[serde(default)]
    current_office: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SavedState {
    #[serde(default)]
    pub desks: Vec<SavedDesk>,
    #[serde(default)]
    pub selected_desk: usize,
}

pub fn save_state(app: &App) -> Result<()> {
    let state = SavedState {
        desks: app
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
        selected_desk: app.selected(),
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

    // Try new format first
    if let Ok(state) = serde_json::from_str::<SavedState>(&json) {
        if !state.desks.is_empty() {
            return Ok(state);
        }
    }

    // Try legacy format (offices) for backward compatibility
    if let Ok(legacy) = serde_json::from_str::<LegacySavedState>(&json) {
        if !legacy.offices.is_empty() {
            let office_idx = legacy
                .current_office
                .min(legacy.offices.len().saturating_sub(1));
            let office = legacy.offices.into_iter().nth(office_idx).unwrap();
            return Ok(SavedState {
                selected_desk: office.active_desk,
                desks: office.desks,
            });
        }
    }

    Err(MatoError::StateParseFailed(format!(
        "Invalid state file: {}",
        path.display()
    )))
}
