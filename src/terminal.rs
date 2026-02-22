use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

use crossterm::{
    cursor::{self, MoveTo},
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, Clear, ClearType, LeaveAlternateScreen},
};

static RESUMED: AtomicBool = AtomicBool::new(false);
static PANIC_HOOK_ONCE: Once = Once::new();
#[cfg(unix)]
static SIGNAL_HOOK_ONCE: Once = Once::new();

pub fn restore_terminal_modes() {
    let mut stdout = std::io::stdout();
    disable_raw_mode().ok();
    execute!(
        stdout,
        Clear(ClearType::All),
        MoveTo(0, 0),
        LeaveAlternateScreen,
        DisableMouseCapture,
        crossterm::event::DisableBracketedPaste,
        cursor::Show
    )
    .ok();
    stdout
        .write_all(b"\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1015l\x1b[?2004l")
        .ok();
    stdout.flush().ok();
}

pub fn consume_resumed() -> bool {
    RESUMED.swap(false, Ordering::Relaxed)
}

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> Self {
        install_panic_cleanup();
        install_signal_cleanup();
        // Reset stale modes in case a previous run crashed.
        restore_terminal_modes();
        Self
    }
}

impl Default for TerminalGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal_modes();
    }
}

fn install_panic_cleanup() {
    PANIC_HOOK_ONCE.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore_terminal_modes();
            previous(info);
        }));
    });
}

fn install_signal_cleanup() {
    #[cfg(unix)]
    {
        use signal_hook::{
            consts::signal::{SIGCONT, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGTSTP},
            iterator::Signals,
            low_level::emulate_default_handler,
        };

        SIGNAL_HOOK_ONCE.call_once(|| {
            let mut signals = Signals::new([SIGCONT, SIGTERM, SIGINT, SIGHUP, SIGQUIT, SIGTSTP])
                .expect("failed to register signal handlers");
            std::thread::spawn(move || {
                for signal in signals.forever() {
                    match signal {
                        SIGCONT => RESUMED.store(true, Ordering::Relaxed),
                        SIGTERM | SIGINT | SIGHUP | SIGQUIT | SIGTSTP => {
                            restore_terminal_modes();
                            let _ = emulate_default_handler(signal);
                        }
                        _ => {}
                    }
                }
            });
        });
    }
}
