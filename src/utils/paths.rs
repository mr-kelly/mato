use std::path::PathBuf;

pub fn get_socket_path() -> PathBuf {
    state_dir().join("daemon.sock")
}

pub fn get_log_path() -> PathBuf {
    state_dir().join("daemon.log")
}

pub fn get_client_log_path() -> PathBuf {
    state_dir().join("client.log")
}

pub fn get_lock_path() -> PathBuf {
    state_dir().join("daemon.lock")
}

pub fn get_pid_path() -> PathBuf {
    state_dir().join("daemon.pid")
}

pub fn get_state_file_path() -> PathBuf {
    config_dir().join("state.json")
}

pub fn get_config_file_path() -> PathBuf {
    config_dir().join("config.toml")
}

fn state_dir() -> PathBuf {
    if let Some(dirs) = directories::ProjectDirs::from("", "", "mato") {
        let dir = dirs.state_dir().unwrap_or(dirs.config_dir());
        std::fs::create_dir_all(dir).ok();
        dir.to_path_buf()
    } else {
        PathBuf::from("/tmp/mato")
    }
}

fn config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut h = PathBuf::from(std::env::var("HOME").unwrap_or_default());
            h.push(".config");
            h
        })
        .join("mato")
}
