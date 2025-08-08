use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;

use org_pulse::app::{events::handle_events, state::App, ui::ui};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state and load data
    let mut app = match App::new_with_data().await {
        Ok(app) => app,
        Err(e) => {
            // Restore terminal before showing error
            disable_raw_mode()?;
            execute!(
                io::stdout(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            eprintln!("Failed to initialize app with data: {}", e);
            return Ok(());
        }
    };

    // Run the TUI
    let result = run_tui(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_tui<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui(f, app))?;

        // Handle events
        handle_events(app)?;

        // Handle pending view switches
        app.handle_pending_view_switch().await?;

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
