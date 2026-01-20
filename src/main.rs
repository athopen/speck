//! spec-tui: Terminal UI for spec-driven development workflow
//!
//! A keyboard-driven TUI for managing feature specifications with
//! git worktree integration for parallel development.

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::path::PathBuf;
use std::panic;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use spec_tui::App;

/// Setup the terminal for TUI mode
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to normal mode
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Initialize logging with RUST_LOG environment variable support
fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
        .init();
}

/// Install a panic hook that restores the terminal before printing the panic
fn install_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal on panic
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    // Install panic hook for graceful terminal restoration
    install_panic_hook();

    // Find project root
    let project_root = spec_tui::domain::Project::discover(None)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    tracing::info!("Starting spec-tui in {:?}", project_root);

    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Create and run app with Ctrl+C handling
    let result = {
        let mut app = App::new(project_root)?;

        // Run with Ctrl+C signal handling
        tokio::select! {
            res = app.run(&mut terminal) => res,
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Received Ctrl+C, shutting down gracefully");
                Ok(())
            }
        }
    };

    // Restore terminal (always, even on error)
    restore_terminal(&mut terminal)?;

    // Handle result
    result?;

    Ok(())
}
