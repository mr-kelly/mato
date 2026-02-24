// Library exports for testing
pub mod client;
pub mod config;
pub mod daemon;
pub mod emulators;
pub mod error;
pub mod passthrough;
pub mod protocol;
pub mod providers;
pub mod terminal;
pub mod terminal_emulator;
pub mod terminal_provider;
pub mod theme;
pub mod utils;

// Re-export commonly used items
pub use config::Config;
pub use error::{MatoError, Result};
pub use protocol::{ClientMsg, ServerMsg};
