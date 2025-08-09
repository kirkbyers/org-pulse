# Org Pulse

A GitHub organization statistics application with an interactive Terminal User Interface (TUI) for browsing and analyzing contribution data across your engineering organization.

## Features

- **Interactive TUI**: Browse organizations, repositories, and contributors with keyboard navigation
- **Multi-view Analysis**: Switch between organization, repository, and contributor perspectives
- **Drill-down Navigation**: Explore from orgs → repos → contributors → contributions
- **Historical Data**: Browse different scrapes and compare data over time
- **Live Scraping**: Start new data collection directly from the TUI
- **Flexible Sorting**: Sort by commits, lines of code, repositories, or contributor counts

## Config

### Settings

a `config.toml` is generated in the directory the application is ran in.

### Github Token

Use `gh` to set github token to use

```bash
$ export GITHUB_TOKEN=$(gh auth token)
```

## Usage

### Starting the Application

After setting up your GitHub token, simply run:

```bash
cargo run
```

The TUI will start and automatically load the most recent scrape data. If no data exists, press `S` to start your first scrape.

### Keyboard Shortcuts

#### Navigation
- `↑/↓` or `j/k` - Navigate up/down through items
- `Enter` - Drill down into selected item (org → repos → contributors → contributions)
- `Escape` - Go back to previous view
- `q` - Quit application

#### View Switching
- `o` - Switch to Organizations view
- `r` - Switch to Repositories view  
- `u` - Switch to Contributors/Users view

#### Sorting
- `s` - Sort by current field (toggle ascending/descending)
- `n` - Sort by name
- `c` - Sort by commits
- `l` - Sort by lines of code
- `p` - Sort by PRs (repository view only)
- `R` - Sort by repository count (org/contributor views)

#### Data Management
- `t` - Browse and select different scrapes
- `S` - Start new scrape (collects fresh data)
- `F5` - Refresh current view

### Usage Examples

1. **Browse Organization Data**:
   - Start the app, press `o` for organizations view
   - Use arrow keys to select an organization
   - Press `Enter` to see all repositories in that org

2. **Find Top Contributors**:
   - Press `u` for contributors view
   - Press `c` to sort by commits, or `l` to sort by lines
   - Press `s` to toggle between ascending/descending

3. **Analyze Repository Activity**:
   - Press `r` for repositories view
   - Press `Enter` on a repo to see its contributors
   - Press `Enter` on a contributor to see their contributions to that repo

4. **Compare Historical Data**:
   - Press `t` to browse available scrapes
   - Select different time periods to compare activity

5. **Collect Fresh Data**:
   - Press `S` to start a new scrape
   - Wait for completion (shows "SCRAPING..." indicator)
   - Data automatically refreshes when complete

## Who is this for

Members of a growing engineering org or startup

- The ICs that are suffering from imposter syndrom
  - Take a look at you contributions in the repos you work in relative 
    to the rest of the org
- The managers that can't keep up with various projects
  - Look back on the week and see exactly what progress was made where
- The leaders trying to wrangle focus and direction
  - Where are efforts going

## Why this application

Developer productivity isn't messurable by a single statistic. However, 
by looking at repository statistics you can begin to see patterns and
at the very least outliers in large swings in performance.

Org Pulse provides a light solution to give you answers and insights 
without having to commit to an entirely new methodology or process.

Org Pulse is designed for you to outgrow eventually, but until you reach
organizational maturity, this project aims to help you identify strugling projects,
high velocity inatives, and celebrate efforts.

Its simple. A weekly summary containing

- contributors by commit counts on main
- contributors by LoC changed
- count of repos contributed by each user

per repo you have access to.
