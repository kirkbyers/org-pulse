use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use anyhow::Result;

use super::state::{App, View, SortField};

pub fn handle_events(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            handle_key_event(key, app)?;
        }
    }
    Ok(())
}

fn handle_key_event(key: KeyEvent, app: &mut App) -> Result<()> {
    if key.kind != KeyEventKind::Press {
        return Ok(());
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
        KeyCode::Char('o') => app.switch_view(View::Org),
        KeyCode::Char('r') => app.switch_view(View::Repo),
        KeyCode::Char('u') => app.switch_view(View::Contributors),
        KeyCode::Up | KeyCode::Char('k') => app.move_selection_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_selection_down(),
        KeyCode::Char('s') => app.toggle_sort_order(),
        KeyCode::Char('n') => app.set_sort_field(SortField::Name),
        KeyCode::Char('c') => app.set_sort_field(SortField::Commits),
        KeyCode::Char('l') => app.set_sort_field(SortField::Lines),
        KeyCode::Char('p') => app.set_sort_field(SortField::Prs),
        KeyCode::Char('R') => app.set_sort_field(SortField::Repos),
        KeyCode::Char('S') => {
            // TODO: Start new scrape
            app.is_scraping = true;
        }
        KeyCode::Enter => {
            // TODO: Handle drill-down navigation
        }
        _ => {}
    }
    Ok(())
}