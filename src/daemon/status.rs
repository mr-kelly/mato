use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::process::Command;
use std::time::SystemTime;
use crate::client::persistence::SavedState;
use crate::protocol::{ClientMsg, ServerMsg};

pub fn show_status() -> io::Result<()> {
    use std::os::unix::net::UnixStream;
    
    let socket_path = crate::utils::get_socket_path();
    let log_path = crate::utils::get_log_path();
    let pid_path = crate::utils::get_pid_path();
    let state_path = crate::utils::get_state_file_path();
    let config_path = crate::utils::get_config_file_path();
    
    let mut process_status: Option<Vec<(String, u32)>> = None;
    let mut daemon_pid: Option<u32> = None;

    println!("Mato Status");
    println!("═══════════════════════════════════════════════════════");
    
    // Daemon status
    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            println!("✅ Daemon:        Running");
            
            // Show PID
            if let Some(pid) = super::PidFile::read(&pid_path) {
                daemon_pid = Some(pid);
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
            process_status = query_process_status(&mut stream).ok();
        }
        Err(_) => {
            println!("❌ Daemon:        Not running");
            if socket_path.exists() {
                println!("   Note:          Stale socket will be cleaned on next start");
            }
        }
    }
    
    println!();

    if let Some(pid) = daemon_pid {
        if let Some((cpu, rss_kb)) = query_process_usage(pid) {
            println!("🧠 Daemon Usage:");
            println!("   CPU:           {:.1}%", cpu);
            println!("   Memory:        {}", format_size(rss_kb * 1024));
            println!();
        }
    }
    
    let mut tab_names: HashMap<String, String> = HashMap::new();

    // Workspace status
    let mut state_opt: Option<SavedState> = None;
    if state_path.exists() {
        println!("📁 Workspace:     Configured");
        
        // Parse state file to show office/desk/tab counts
        if let Ok(content) = fs::read_to_string(&state_path) {
            if let Ok(state) = serde_json::from_str::<SavedState>(&content) {
                state_opt = Some(state);
            }
        }

        if let Some(state) = state_opt.as_ref() {
                tab_names = tab_name_map(state);
                let office_count = state.offices.len();
                let desk_count: usize = state.offices.iter().map(|o| o.desks.len()).sum();
                let total_tabs: usize = state.offices.iter()
                    .flat_map(|o| o.desks.iter())
                    .map(|d| d.tabs.len())
                    .sum();

                println!("   Offices:       {}", office_count);
                println!("   Desks:         {}", desk_count);
                println!("   Total Tabs:    {}", total_tabs);
                match process_status.as_ref() {
                    Some(tabs) => {
                        println!("   Running TTYs:  {}/{}", tabs.len(), total_tabs);
                    }
                    None if daemon_pid.is_some() => println!("   Running TTYs:  unavailable"),
                    None => {}
                }

                if let Some(active_office) = state.offices.get(state.current_office) {
                    println!("   Active Office: {}", active_office.name);
                    if let Some(active_desk) = active_office.desks.get(active_office.active_desk) {
                        println!("   Active Desk:   {}", active_desk.name);
                    }
                }
        }
        
        println!("   State:         {}", state_path.display());
    } else {
        println!("📁 Workspace:     Not configured (first run)");
    }
    
    println!();

    if let Some(proc_tabs) = process_status.as_ref() {
        let pids: Vec<u32> = proc_tabs.iter().map(|(_, pid)| *pid).collect();
        let usage = query_process_usage_map(&pids);

        let mut total_cpu = 0.0_f64;
        let mut total_rss_kb = 0_u64;
        let mut details: Vec<(String, u32, f64, u64)> = vec![];
        for (tab_id, pid) in proc_tabs {
            if let Some((cpu, rss_kb)) = usage.get(pid).copied() {
                total_cpu += cpu;
                total_rss_kb += rss_kb;
                let label = tab_names.get(tab_id).cloned().unwrap_or_else(|| tab_id.clone());
                details.push((label, *pid, cpu, rss_kb));
            }
        }
        details.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        println!("🖥️  TTY Processes:");
        println!("   Count:         {}", proc_tabs.len());
        println!("   Total CPU:     {:.1}%", total_cpu);
        println!("   Total Memory:  {}", format_size(total_rss_kb * 1024));
        for (label, pid, cpu, rss_kb) in details.into_iter().take(5) {
            println!("   - {} (pid {}, cpu {:.1}%, mem {})", label, pid, cpu, format_size(rss_kb * 1024));
        }
        if proc_tabs.len() > 5 {
            println!("   ... and {} more", proc_tabs.len() - 5);
        }
        println!();
    }
    
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

fn query_process_status(stream: &mut std::os::unix::net::UnixStream) -> io::Result<Vec<(String, u32)>> {
    let msg = serde_json::to_vec(&ClientMsg::GetProcessStatus)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    stream.write_all(&msg)?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    if line.trim().is_empty() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "empty daemon response"));
    }

    match serde_json::from_str::<ServerMsg>(line.trim_end())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?
    {
        ServerMsg::ProcessStatus { tabs } => Ok(tabs),
        ServerMsg::Error { message } => Err(io::Error::other(message)),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected daemon response")),
    }
}

fn tab_name_map(state: &SavedState) -> HashMap<String, String> {
    let mut names = HashMap::new();
    for office in &state.offices {
        for desk in &office.desks {
            for tab in &desk.tabs {
                names.insert(
                    tab.id.clone(),
                    format!("{}/{}/{}", office.name, desk.name, tab.name),
                );
            }
        }
    }
    names
}

fn query_process_usage(pid: u32) -> Option<(f64, u64)> {
    let output = Command::new("ps")
        .args(["-o", "%cpu=,rss=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().find(|l| !l.trim().is_empty())?;
    let mut parts = line.split_whitespace();
    let cpu = parts.next()?.parse::<f64>().ok()?;
    let rss_kb = parts.next()?.parse::<u64>().ok()?;
    Some((cpu, rss_kb))
}

fn query_process_usage_map(pids: &[u32]) -> HashMap<u32, (f64, u64)> {
    if pids.is_empty() {
        return HashMap::new();
    }

    let pid_list = pids.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",");
    let output = match Command::new("ps")
        .args(["-o", "pid=,%cpu=,rss=", "-p", &pid_list])
        .output()
    {
        Ok(out) if out.status.success() => out,
        _ => return HashMap::new(),
    };

    let mut out = HashMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        if let (Ok(pid), Ok(cpu), Ok(rss_kb)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<f64>(),
            parts[2].parse::<u64>(),
        ) {
            out.insert(pid, (cpu, rss_kb));
        }
    }
    out
}
