use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

pub struct DaemonLock {
    _file: File,
    path: PathBuf,
}

impl DaemonLock {
    pub fn acquire(path: PathBuf) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;

        #[cfg(unix)]
        {
            use libc::{flock, LOCK_EX, LOCK_NB};
            let fd = file.as_raw_fd();
            if unsafe { flock(fd, LOCK_EX | LOCK_NB) } != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Daemon already running (lock file held)",
                ));
            }
        }

        // Write PID to lock file
        let mut f = &file;
        write!(f, "{}", std::process::id())?;
        f.flush()?;

        Ok(Self { _file: file, path })
    }
}

impl Drop for DaemonLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
