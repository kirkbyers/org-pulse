use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::state::{App, View, ViewData, SortField, SortOrder};

pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(2), // Footer
        ])
        .split(f.size());

    draw_header(f, chunks[0], app);
    draw_main_content(f, chunks[1], app);
    draw_footer(f, chunks[2], app);
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let scrape_info = match app.current_scrape {
        Some(id) => format!("Scrape ID: {}", id),
        None => "No scrape selected".to_string(),
    };
    
    let view_name = match app.current_view {
        View::Org => "Organizations",
        View::Repo => "Repositories", 
        View::Contributors => "Contributors",
    };

    let header_text = format!("{} | {}", scrape_info, view_name);
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("org-pulse TUI"))
        .alignment(Alignment::Center);
    
    f.render_widget(header, area);
}

fn draw_main_content(f: &mut Frame, area: Rect, app: &App) {
    match &app.data {
        ViewData::Loading => {
            let loading = Paragraph::new("Loading...")
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center);
            f.render_widget(loading, area);
        }
        ViewData::Error(msg) => {
            let error = Paragraph::new(format!("Error: {}", msg))
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);
            f.render_widget(error, area);
        }
        ViewData::Orgs(_) => draw_org_table(f, area, app),
        ViewData::Repos(_) => draw_repo_table(f, area, app),
        ViewData::Contributors(_) => draw_contributor_table(f, area, app),
    }
}

fn draw_org_table(f: &mut Frame, area: Rect, _app: &App) {
    // TODO: Implement org table rendering
    let placeholder = Paragraph::new("Organizations view (TODO)")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(placeholder, area);
}

fn draw_repo_table(f: &mut Frame, area: Rect, _app: &App) {
    // TODO: Implement repo table rendering  
    let placeholder = Paragraph::new("Repositories view (TODO)")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(placeholder, area);
}

fn draw_contributor_table(f: &mut Frame, area: Rect, _app: &App) {
    // TODO: Implement contributor table rendering
    let placeholder = Paragraph::new("Contributors view (TODO)")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(placeholder, area);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let sort_info = format!(
        "Sort: {} {}",
        match app.sort_field {
            SortField::Name => "Name",
            SortField::Commits => "Commits",
            SortField::Lines => "Lines", 
            SortField::Repos => "Repos",
            SortField::Prs => "PRs",
        },
        match app.sort_order {
            SortOrder::Ascending => "↑",
            SortOrder::Descending => "↓",
        }
    );

    let footer_text = format!("q: Quit | o: Orgs | r: Repos | u: Contributors | {}", sort_info);
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);
    
    f.render_widget(footer, area);
}