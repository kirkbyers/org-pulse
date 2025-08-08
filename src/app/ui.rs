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
            Constraint::Length(3), // Footer
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
        View::ScrapeSelection => "Scrape Selection",
    };

    let (_item_count, selection_info) = match app.current_view {
        View::ScrapeSelection => {
            let count = app.scrapes.len();
            if count > 0 {
                (count, format!(" | {}/{} scrapes", app.scrape_selected_index + 1, count))
            } else {
                (0, " | No scrapes".to_string())
            }
        }
        _ => {
            let count = app.get_item_count();
            if count > 0 {
                (count, format!(" | {}/{} items", app.selected_index + 1, count))
            } else {
                (0, " | No items".to_string())
            }
        }
    };

    let status_text = if app.is_scraping {
        " | 🔄 SCRAPING..."
    } else if let Some(error) = &app.scraping_error {
        &format!(" | ❌ ERROR: {}", error)
    } else {
        ""
    };

    let header_text = format!("{} | {}{}{}", scrape_info, view_name, selection_info, status_text);
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("org-pulse TUI"))
        .alignment(Alignment::Center);
    
    f.render_widget(header, area);
}

fn draw_main_content(f: &mut Frame, area: Rect, app: &App) {
    // Show scraping overlay when scraping is in progress
    if app.is_scraping {
        let scraping = Paragraph::new("🔄 SCRAPING IN PROGRESS...\n\nPlease wait while fetching GitHub data.\nThis may take several minutes depending on organization size.")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(scraping, area);
        return;
    }

    match app.current_view {
        View::ScrapeSelection => draw_scrape_selection_table(f, area, app),
        _ => match &app.data {
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

fn draw_scrape_selection_table(f: &mut Frame, area: Rect, app: &App) {
    if app.scrapes.is_empty() {
        let placeholder = Paragraph::new("No scrapes available")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(placeholder, area);
        return;
    }

    let header_cells = ["Scrape ID", "Start Date", "End Date", "Repository Count"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .height(1);

    let rows: Vec<Row> = app.scrapes.iter().enumerate().map(|(i, scrape)| {
        let cells = vec![
            Cell::from(scrape.id.to_string()),
            Cell::from(scrape.start_dt.format("%Y-%m-%d %H:%M").to_string()),
            Cell::from(scrape.end_dt.format("%Y-%m-%d %H:%M").to_string()),
            Cell::from(scrape.repo_count.to_string()),
        ];
        let mut row = Row::new(cells).height(1);
        if i == app.scrape_selected_index {
            row = row.style(Style::default().bg(Color::DarkGray).fg(Color::White));
        }
        row
    }).collect();

    let table = Table::new(
        rows,
        &[
            Constraint::Percentage(15), // Scrape ID
            Constraint::Percentage(30), // Start Date
            Constraint::Percentage(30), // End Date
            Constraint::Percentage(25), // Repository Count
        ]
    )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Available Scrapes"))
        .column_spacing(1);

    f.render_widget(table, area);
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

    // Split footer into two lines for better readability
    let footer_line1 = if app.current_view == View::ScrapeSelection {
        "Navigation: ↑↓/j/k | Enter: Select | Esc/t: Back | q: Quit"
    } else {
        "Navigation: ↑↓/j/k | Views: o/r/u | t: Scrapes | Sort: s/n/c/l/p/R | S: New Scrape | q: Quit"
    };
    let footer_line2 = format!("{}", sort_info);
    
    let footer_text = format!("{}\n{}", footer_line1, footer_line2);
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);
    
    f.render_widget(footer, area);
}