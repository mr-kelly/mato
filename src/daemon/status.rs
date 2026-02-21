use std::io;
use std::fs;
use std::time::SystemTime;
use crate::client::persistence::SavedState;

pub fn show_status() -> io::Result<()> {
    use std::os::unix::net::UnixStream;
    
    let socket_path = crate::utils::get_socket_path();
    let log_path = crate::utils::get_log_path();
    let pid_path = crate::utils::get_pid_path();
    let state_path = crate::utils::get_state_file_path();
    let config_path = crate::utils::get_config_file_path();
    
    println!("Mato Status");
    println!("═══════════════════════════════════════════════════════");
    
    // Daemon status
    match UnixStream::connect(&socket_path) {
        Ok(_) => {
            println!("✅ Daemon:        Running");
            
            // Show PID
            if let Some(pid) = super::PidFile::read(&pid_path) {
                println!("   PID:           {}", pid);
            }
            
            // Show uptime
            if let Ok(metadata) = fs::metadata(&socket_path) {
                if let Ok(created) = metadata.modified() {
                    if let Ok(duration) = SystemTime::now().duration_since(created) {
                        let secs = duration.as_secs();
                        let uptime = if secs < 60 {
                            format!("{}s", secs)
                        } else if secs < 3600 {
                            format!("{}m {}s", secs / 60, secs % 60)
                        } else {
                            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
                        };
                        println!("   Uptime:        {}", uptime);
                    }
                }
            }
            
            println!("   Socket:        {}", socket_path.display());
        }
        Err(_) => {
            println!("❌ Daemon:        Not running");
            if socket_path.exists() {
                println!("   Note:          Stale socket will be cleaned on next start");
            }
        }
    }
    
    println!();
    
    // Workspace status
    if state_path.exists() {
        println!("📁 Workspace:     Configured");
        
        // Parse state file to show office/desk/tab counts
        if let Ok(content) = fs::read_to_string(&state_path) {
            if let Ok(state) = serde_json::from_str::<SavedState>(&content) {
                let office_count = state.offices.len();
                let desk_count: usize = state.offices.iter().map(|o| o.desks.len()).sum();
                let total_tabs: usize = state.offices.iter()
                    .flat_map(|o| o.desks.iter())
                    .map(|d| d.tabs.len())
                    .sum();

                println!("   Offices:       {}", office_count);
                println!("   Desks:         {}", desk_count);
                println!("   Total Tabs:    {}", total_tabs);

                if let Some(active_office) = state.offices.get(state.current_office) {
                    println!("   Active Office: {}", active_office.name);
                    if let Some(active_desk) = active_office.desks.get(active_office.active_desk) {
                        println!("   Active Desk:   {}", active_desk.name);
                    }
                }
            }
        }
        
        println!("   State:         {}", state_path.display());
    } else {
        println!("📁 Workspace:     Not configured (first run)");
    }
    
    println!();
    
    // Configuration
    if config_path.exists() {
        println!("⚙️  Config:        {}", config_path.display());
        
        // Show emulator setting
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                if let Some(emulator) = config.get("emulator").and_then(|v| v.as_str()) {
                    println!("   Emulator:      {}", emulator);
                }
            }
        }
    } else {
        println!("⚙️  Config:        Using defaults (vte)");
    }
    
    println!();
    
    // Logs
    println!("📝 Logs:");
    if log_path.exists() {
        if let Ok(metadata) = fs::metadata(&log_path) {
            let size = format_size(metadata.len());
            println!("   Daemon:        {} ({})", log_path.display(), size);
        }
    }
    
    println!("═══════════════════════════════════════════════════════");
    
    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
