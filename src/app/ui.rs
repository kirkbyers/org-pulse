use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, Cell},
    Frame,
};

use super::state::{App, View, SortField, SortOrder};
use crate::stats::ViewData;

fn format_number(num: i64) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}

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

fn draw_org_table(f: &mut Frame, area: Rect, app: &App) {
    if let ViewData::Orgs(orgs) = &app.data {
        let header_cells = ["Organization", "Commits", "Lines", "Repos", "Contributors"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
        
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::Blue).fg(Color::White))
            .height(1);

        let rows: Vec<Row> = orgs.iter().enumerate().map(|(i, org)| {
            let cells = vec![
                Cell::from(org.name.clone()),
                Cell::from(format_number(org.total_commits)),
                Cell::from(format_number(org.total_lines)),
                Cell::from(format_number(org.repo_count)),
                Cell::from(format_number(org.contributor_count)),
            ];
            let mut row = Row::new(cells).height(1);
            if i == app.selected_index {
                row = row.style(Style::default().bg(Color::DarkGray).fg(Color::White));
            }
            row
        }).collect();

        let table = Table::new(
            rows,
            &[
                Constraint::Percentage(30), // Organization name
                Constraint::Percentage(15), // Commits
                Constraint::Percentage(15), // Lines 
                Constraint::Percentage(15), // Repos
                Constraint::Percentage(25), // Contributors
            ]
        )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Organizations"))
            .column_spacing(1);

        f.render_widget(table, area);
    } else {
        let placeholder = Paragraph::new("No organization data")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(placeholder, area);
    }
}

fn draw_repo_table(f: &mut Frame, area: Rect, app: &App) {
    if let ViewData::Repos(repos) = &app.data {
        let header_cells = ["Organization", "Repository", "Commits", "Lines", "PRs", "Contributors"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
        
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::Blue).fg(Color::White))
            .height(1);

        let rows: Vec<Row> = repos.iter().enumerate().map(|(i, repo)| {
            let cells = vec![
                Cell::from(repo.org_name.clone()),
                Cell::from(repo.repo_name.clone()),
                Cell::from(format_number(repo.commits)),
                Cell::from(format_number(repo.lines)),
                Cell::from(format_number(repo.prs)),
                Cell::from(format_number(repo.contributor_count)),
            ];
            let mut row = Row::new(cells).height(1);
            if i == app.selected_index {
                row = row.style(Style::default().bg(Color::DarkGray).fg(Color::White));
            }
            row
        }).collect();

        let table = Table::new(
            rows,
            &[
                Constraint::Percentage(20), // Organization name
                Constraint::Percentage(25), // Repository name
                Constraint::Percentage(12), // Commits
                Constraint::Percentage(12), // Lines 
                Constraint::Percentage(12), // PRs
                Constraint::Percentage(19), // Contributors
            ]
        )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Repositories"))
            .column_spacing(1);

        f.render_widget(table, area);
    } else {
        let placeholder = Paragraph::new("No repository data")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(placeholder, area);
    }
}

fn draw_contributor_table(f: &mut Frame, area: Rect, app: &App) {
    if let ViewData::Contributors(contributors) = &app.data {
        let header_cells = ["Username", "Commits", "Lines", "Repos", "Organizations"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
        
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::Blue).fg(Color::White))
            .height(1);

        let rows: Vec<Row> = contributors.iter().enumerate().map(|(i, contributor)| {
            let orgs_display = if contributor.orgs.len() <= 2 {
                contributor.orgs.join(", ")
            } else {
                format!("{}, {} (+{})", 
                    contributor.orgs[0], 
                    contributor.orgs[1], 
                    contributor.orgs.len() - 2)
            };

            let cells = vec![
                Cell::from(contributor.username.clone()),
                Cell::from(format_number(contributor.total_commits)),
                Cell::from(format_number(contributor.total_lines)),
                Cell::from(format_number(contributor.repo_count)),
                Cell::from(orgs_display),
            ];
            let mut row = Row::new(cells).height(1);
            if i == app.selected_index {
                row = row.style(Style::default().bg(Color::DarkGray).fg(Color::White));
            }
            row
        }).collect();

        let table = Table::new(
            rows,
            &[
                Constraint::Percentage(20), // Username
                Constraint::Percentage(15), // Commits
                Constraint::Percentage(15), // Lines 
                Constraint::Percentage(15), // Repos
                Constraint::Percentage(35), // Organizations
            ]
        )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Contributors"))
            .column_spacing(1);

        f.render_widget(table, area);
    } else {
        let placeholder = Paragraph::new("No contributor data")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(placeholder, area);
    }
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

    let footer_text = format!("q: Quit | ↑↓/j/k: Navigate | o: Orgs | r: Repos | u: Contributors | {}", sort_info);
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);
    
    f.render_widget(footer, area);
}