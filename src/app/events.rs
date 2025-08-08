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
        KeyCode::Char('q') => app.quit(),
        KeyCode::Esc => {
            // Exit scrape selection mode back to data view
            if app.current_view == View::ScrapeSelection {
                app.request_view_switch(View::Org);
            } else {
                app.quit();
            }
        }
        KeyCode::Char('t') => {
            // Toggle scrape selection mode
            if app.current_view == View::ScrapeSelection {
                app.request_view_switch(View::Org);
            } else {
                app.request_view_switch(View::ScrapeSelection);
            }
        }
        KeyCode::Char('o') => app.request_view_switch(View::Org),
        KeyCode::Char('r') => app.request_view_switch(View::Repo),
        KeyCode::Char('u') => app.request_view_switch(View::Contributors),
        KeyCode::Up | KeyCode::Char('k') => {
            if app.current_view == View::ScrapeSelection {
                app.move_scrape_selection_up();
            } else {
                app.move_selection_up();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.current_view == View::ScrapeSelection {
                app.move_scrape_selection_down();
            } else {
                app.move_selection_down();
            }
        }
        KeyCode::Enter => {
            if app.current_view == View::ScrapeSelection {
                // Select the current scrape and return to org view
                // This will be handled in the main loop as it's async
                app.pending_view_switch = Some(View::Org);
            }
            // TODO: Handle drill-down navigation for other views
        }
        KeyCode::Char('s') => {
            if app.current_view != View::ScrapeSelection {
                app.toggle_sort_order();
            }
        }
        KeyCode::Char('n') => {
            if app.current_view != View::ScrapeSelection {
                app.set_sort_field(SortField::Name);
            }
        }
        KeyCode::Char('c') => {
            if app.current_view != View::ScrapeSelection {
                app.set_sort_field(SortField::Commits);
            }
        }
        KeyCode::Char('l') => {
            if app.current_view != View::ScrapeSelection {
                app.set_sort_field(SortField::Lines);
            }
        }
        KeyCode::Char('p') => {
            if app.current_view != View::ScrapeSelection {
                app.set_sort_field(SortField::Prs);
            }
        }
        KeyCode::Char('R') => {
            if app.current_view != View::ScrapeSelection {
                app.set_sort_field(SortField::Repos);
            }
        }
        KeyCode::Char('S') => {
            // Don't start new scrape if already scraping
            if !app.is_scraping {
                app.request_scraping();
            }
        }
        _ => {}
    }
    Ok(())
}