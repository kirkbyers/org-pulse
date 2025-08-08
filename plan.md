# Ratatui TUI Implementation Plan

## Overview
Transform org-pulse from a command-line scraper into an interactive TUI application for browsing GitHub organization data using ratatui.

## Architecture Changes

### 1. Dependencies
Add to `Cargo.toml`:
```toml
ratatui = "0.26"
crossterm = "0.27"
```

### 2. Project Structure
```
src/
├── main.rs          # TUI entry point
├── app/
│   ├── mod.rs       # App state and main loop
│   ├── ui.rs        # UI rendering
│   ├── events.rs    # Event handling
│   └── state.rs     # Application state management
├── scraper.rs       # Move current main.rs logic here
├── db.rs           # Existing (enhanced with TUI queries)
└── stats.rs        # Statistics calculation utilities
```

## Core Features

### 1. Application State
```rust
pub struct App {
    pub current_view: View,           // Org, Repo, Contributors
    pub current_scrape: Option<i64>,  // Selected scrape ID
    pub scrapes: Vec<ScrapeInfo>,     // Available scrapes
    pub data: ViewData,               // Current view data
    pub sort_order: SortOrder,        // Ascending/Descending
    pub sort_field: SortField,        // What to sort by
    pub selected_index: usize,        // Currently selected item
    pub should_quit: bool,
    pub is_scraping: bool,            // Background scrape status
}

pub enum View {
    Org,
    Repo, 
    Contributors,
}

pub enum SortField {
    Name,
    Commits,
    Lines,
    Repos, // For orgs and contributors
    Prs,   // For repos
}
```

### 2. Statistics Calculation
Create aggregated statistics from database:
- **Per Org**: Total commits, lines, repos, contributors
- **Per Repo**: Total commits, lines, PRs, contributors
- **Per Contributor**: Total commits, lines, repos contributed to

### 3. Database Query Extensions
Add methods to `src/db.rs` for TUI:
```rust
// Scrape listing
impl Scrape {
    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<ScrapeInfo>>;
    pub async fn get_latest(pool: &SqlitePool) -> Result<Option<Scrape>>;
}

// Statistics queries
pub async fn get_org_stats(pool: &SqlitePool, scrape_id: i64) -> Result<Vec<OrgStats>>;
pub async fn get_repo_stats(pool: &SqlitePool, scrape_id: i64) -> Result<Vec<RepoStats>>;
pub async fn get_contributor_stats(pool: &SqlitePool, scrape_id: i64) -> Result<Vec<ContributorStats>>;
```

### 4. UI Layout
Three-panel layout:
- **Header**: Current scrape info, navigation hints
- **Main Panel**: Data table (org/repo/contributors)
- **Footer**: Status bar with sorting info and keyboard shortcuts

### 5. Keyboard Shortcuts
- `q`: Quit application
- `o`: Switch to Organizations view
- `r`: Switch to Repositories view  
- `u`: Switch to Contributors/Users view
- `s`: Sort by current field (toggle asc/desc)
- `n`: Sort by name
- `c`: Sort by commits
- `l`: Sort by lines
- `p`: Sort by PRs (repo view only)
- `R`: Sort by repo count (org/contributor views)
- `S`: Start new scrape (background task)
- `↑/↓` or `j/k`: Navigate items
- `Enter`: View details (future feature)

## Implementation Steps

### Phase 1: Basic TUI Structure
1. **Setup Dependencies**: Add ratatui and crossterm to Cargo.toml
2. **Create App Module**: Basic app state and event loop
3. **Move Scraper Logic**: Extract current main.rs scraping into separate module
4. **Basic UI**: Simple table display with placeholder data

### Phase 2: Database Integration
1. **Extend Database Models**: Add TUI-specific query methods
2. **Statistics Calculation**: Implement aggregation functions
3. **Data Loading**: Load scrape list and default to latest
4. **Table Population**: Display real data in TUI

### Phase 3: Navigation & Views
1. **View Switching**: Implement keyboard shortcuts for view changes
2. **Data Refresh**: Update displayed data when switching views
3. **Selection Handling**: Track selected items, basic navigation

### Phase 4: Sorting & Polish
1. **Sorting Logic**: Implement multi-field sorting
2. **Background Scraping**: Async scrape execution from TUI
3. **Status Updates**: Show scrape progress
4. **Error Handling**: Graceful error display in TUI

## Data Structures

### Statistics Types
```rust
#[derive(Debug, Clone)]
pub struct OrgStats {
    pub name: String,
    pub total_commits: i64,
    pub total_lines: i64,
    pub repo_count: i64,
    pub contributor_count: i64,
}

#[derive(Debug, Clone)]
pub struct RepoStats {
    pub org_name: String,
    pub repo_name: String,
    pub commits: i64,
    pub lines: i64,
    pub prs: i64,
    pub contributor_count: i64,
}

#[derive(Debug, Clone)]
pub struct ContributorStats {
    pub username: String,
    pub total_commits: i64,
    pub total_lines: i64,
    pub repo_count: i64,
    pub orgs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ScrapeInfo {
    pub id: i64,
    pub start_dt: DateTime<Utc>,
    pub end_dt: DateTime<Utc>,
    pub repo_count: i64,
}
```

## Questions & Clarifications

1. **Scrape Selection**: Should users be able to select different historical scrapes from a list, or just browse the latest?
    - Yes. The latest should be selected by defualt, but users should be able to select scrapes from a list.

2. **Real-time Updates**: When a new scrape is running, should the TUI show live progress or just a "scraping..." indicator?
    - Just a "Scraping..." with an animated "...." is good for now.

3. **Detail Views**: When pressing Enter on an item, what additional details should be shown? 
   - Org: Go to the list of Repos from that org
   - Repo: Go to the list of contributors for that repo scrape.
   - Contributor: Show list of RepoContributions the user had.

4. **Filtering**: Should there be search/filter capabilities within each view?
    - not for now

5. **Data Persistence**: Should the TUI remember the last selected view/sort options between runs?
    - No

6. **Error Recovery**: If a scrape fails, should it be retryable from the TUI?
    - no, just show an error message

7. **Configuration**: Should TUI settings (ignored patterns, etc.) be configurable from within the interface?
    - Not for now

8. **Time Range**: Should users be able to specify different time ranges for new scrapes, or stick with the current 7-day window?
    - Not for now.

## Technical Considerations

1. **Database Schema**: Current scrape.rs seems to use a different pattern than db.rs structures - this needs alignment
    - Fix it as part of the scope of this project. db.rs should be responsible for marshalling in and out data into the db; scrapes.rs should call db.rs

2. **Async Handling**: Background scraping while maintaining responsive TUI requires careful async task management
    - The scraping can happen in the forground and block the TUI for now if that simplifies things.

3. **Memory Usage**: For large organizations, consider pagination or virtual scrolling
    - It should be fine for now. We can tackle this issue later.

4. **Cross-platform**: Ensure terminal handling works across different platforms
    - I'll take care of that.