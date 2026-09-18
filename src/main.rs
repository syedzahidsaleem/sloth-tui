//! Sloth TUI binary entry point.

#[cfg(not(target_os = "android"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// RAII guard that restores terminal state upon drop.
pub struct TerminalGuard;

impl TerminalGuard {
    /// Creates a new `TerminalGuard` instance.
    pub fn new() -> Self {
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
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::Show,
            crossterm::terminal::LeaveAlternateScreen
        );
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let _guard = TerminalGuard::new();

    tracing::info!("Sloth TUI starting...");
    println!("Sloth TUI starting...");

    Ok(())
}
