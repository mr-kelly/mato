use std::fmt;
use std::io;

#[derive(Debug)]
pub enum MatoError {
    // IO errors
    Io(io::Error),
    
    // Daemon errors
    DaemonNotRunning,
    DaemonAlreadyRunning,
    DaemonConnectionFailed(String),
    
    // Lock errors
    LockAcquisitionFailed(String),
    
    // Configuration errors
    ConfigLoadFailed(String),
    ConfigParseFailed(String),
    
    // State errors
    StateLoadFailed(String),
    StateParseFailed(String),
    StateSaveFailed(String),
    
    // Protocol errors
    ProtocolError(String),
    SerializationError(String),
    
    // Terminal errors
    TerminalInitFailed(String),
    PtySpawnFailed(String),
}

impl fmt::Display for MatoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatoError::Io(e) => write!(f, "IO error: {}", e),
            
            MatoError::DaemonNotRunning => {
                write!(f, "Daemon is not running. Start it with 'mato'")
            }
            MatoError::DaemonAlreadyRunning => {
                write!(f, "Daemon is already running. Use 'mato --status' to check")
            }
            MatoError::DaemonConnectionFailed(msg) => {
                write!(f, "Failed to connect to daemon: {}", msg)
            }
            
            MatoError::LockAcquisitionFailed(msg) => {
                write!(f, "Failed to acquire lock: {}. Another daemon may be starting", msg)
            }
            
            MatoError::ConfigLoadFailed(msg) => {
                write!(f, "Failed to load config: {}", msg)
            }
            MatoError::ConfigParseFailed(msg) => {
                write!(f, "Failed to parse config: {}. Check your config.toml syntax", msg)
            }
            
            MatoError::StateLoadFailed(msg) => {
                write!(f, "Failed to load state: {}", msg)
            }
            MatoError::StateParseFailed(msg) => {
                write!(f, "Failed to parse state: {}. Your state.json may be corrupted", msg)
            }
            MatoError::StateSaveFailed(msg) => {
                write!(f, "Failed to save state: {}", msg)
            }
            
            MatoError::ProtocolError(msg) => {
                write!(f, "Protocol error: {}", msg)
            }
            MatoError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            
            MatoError::TerminalInitFailed(msg) => {
                write!(f, "Failed to initialize terminal: {}", msg)
            }
            MatoError::PtySpawnFailed(msg) => {
                write!(f, "Failed to spawn PTY: {}", msg)
            }
        }
    }
}

impl std::error::Error for MatoError {}

impl From<io::Error> for MatoError {
    fn from(err: io::Error) -> Self {
        MatoError::Io(err)
    }
}

impl From<serde_json::Error> for MatoError {
    fn from(err: serde_json::Error) -> Self {
        MatoError::SerializationError(err.to_string())
    }
}

impl From<toml::de::Error> for MatoError {
    fn from(err: toml::de::Error) -> Self {
        MatoError::ConfigParseFailed(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, MatoError>;
