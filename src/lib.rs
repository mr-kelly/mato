// Library exports for testing
pub mod protocol;
pub mod config;
pub mod utils;
pub mod terminal_provider;
pub mod terminal_emulator;
pub mod error;
pub mod emulators;
pub mod providers;
pub mod client;
pub mod daemon_modules;

// Re-export commonly used items
pub use protocol::{ClientMsg, ServerMsg};
pub use config::Config;
pub use error::{MatoError, Result};
