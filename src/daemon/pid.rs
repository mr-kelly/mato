use std::fs;
use std::io;
use std::path::PathBuf;

pub struct PidFile {
    path: PathBuf,
}

impl PidFile {
    pub fn create(path: PathBuf) -> io::Result<Self> {
        let pid = std::process::id();
        fs::write(&path, pid.to_string())?;
        Ok(Self { path })
    }

    pub fn read(path: &PathBuf) -> Option<u32> {
        fs::read_to_string(path).ok()?.trim().parse().ok()
    }
}

impl Drop for PidFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
