# TUI Implementation Tasks

## Phase 1: Basic TUI Structure

### Task 1.1: Setup Dependencies ✅
- [x] Add `ratatui = "0.26"` to Cargo.toml
- [x] Add `crossterm = "0.27"` to Cargo.toml  
- [x] Test basic compilation
- [x] Fixed compilation error by adding `Row` import to db.rs

### Task 1.2: Create Project Structure ✅
- [x] Create `src/app/` directory
- [x] Create `src/app/mod.rs` with basic module structure
- [x] Create `src/app/state.rs` with App struct and enums
- [x] Create `src/app/ui.rs` stub
- [x] Create `src/app/events.rs` stub
- [x] Update `src/lib.rs` to include new modules
- [x] Fixed ratatui Frame API usage for 0.26 compatibility
- [x] Removed unused imports to clean up warnings

### Task 1.3: Move Scraper Logic ✅
- [x] Create `src/scraper.rs`
- [x] Move current `main.rs` logic to `scraper::run_scrape()` function
- [x] Keep main.rs as TUI entry point
- [x] Test that scraper module compiles and works
- [x] Updated lib.rs to include scraper module
- [x] Replaced main.rs with complete TUI setup and event loop

### Task 1.4: Basic TUI Application ✅
- [x] Implement basic event loop in `main.rs`
- [x] Create minimal UI with placeholder table
- [x] Add quit functionality ('q' key)
- [x] Test basic TUI runs and can be quit
- [x] All components working together - full TUI infrastructure complete

## Phase 2: Database Integration

### Task 2.1: Fix Database Architecture Issues ✅
- [x] Review scrape.rs vs db.rs patterns
- [x] Move database operations from scrape.rs to use db.rs create/get methods
- [x] Ensure scrape.rs calls db.rs for all database operations
- [x] Test that existing scraping still works with db.rs methods
- [x] Refactored scraper.rs to use proper db.rs create/get methods
- [x] Added Clone derives to db structs for easier usage
- [x] Deprecated old scrape.rs patterns in favor of db.rs

### Task 2.2: Add Statistics Data Structures
- [ ] Add `src/stats.rs`
- [ ] Define `OrgStats`, `RepoStats`, `ContributorStats` structs
- [ ] Define `ScrapeInfo` struct
- [ ] Add `ViewData` enum to hold different stat types

### Task 2.3: Extend Database with TUI Queries
- [ ] Add `Scrape::list_all()` method to get all scrapes
- [ ] Add `Scrape::get_latest()` method to get most recent scrape
- [ ] Add `get_org_stats(pool, scrape_id)` function
- [ ] Add `get_repo_stats(pool, scrape_id)` function  
- [ ] Add `get_contributor_stats(pool, scrape_id)` function
- [ ] Test all new database functions

### Task 2.4: Data Loading
- [ ] Implement data loading in App initialization
- [ ] Load scrape list on startup
- [ ] Default to latest scrape
- [ ] Load initial org stats view
- [ ] Test data loads correctly in TUI

## Phase 3: Core TUI Functionality

### Task 3.1: View Management
- [ ] Implement view switching logic in app state
- [ ] Add keyboard handlers for 'o', 'r', 'u' keys
- [ ] Update UI to show current view
- [ ] Refresh data when switching views
- [ ] Test view switching works

### Task 3.2: Table Rendering
- [ ] Implement org stats table in UI
- [ ] Implement repo stats table in UI
- [ ] Implement contributor stats table in UI
- [ ] Add table headers and formatting
- [ ] Test all three views display correctly

### Task 3.3: Navigation
- [ ] Add selection index to app state
- [ ] Implement up/down arrow and j/k navigation
- [ ] Highlight selected row in tables
- [ ] Handle wrap-around at list boundaries
- [ ] Test navigation works in all views

### Task 3.4: Header and Footer
- [ ] Add header with current scrape info and view name
- [ ] Add footer with keyboard shortcuts
- [ ] Show current sort field and order
- [ ] Test header/footer display correctly

## Phase 4: Sorting and Polish

### Task 4.1: Sorting Implementation
- [ ] Add sort logic to app state
- [ ] Implement keyboard handlers for sort keys (s, n, c, l, p, R)
- [ ] Sort data when sort field/order changes
- [ ] Update UI to show current sort
- [ ] Test sorting works for all fields in all views

### Task 4.2: Scrape Selection
- [ ] Add scrape selection mode/view
- [ ] Allow browsing available scrapes
- [ ] Switch between scrapes and reload data
- [ ] Add keyboard shortcut to enter scrape selection
- [ ] Test scrape selection works

### Task 4.3: Background Scraping
- [ ] Add "Start Scrape" functionality ('S' key)
- [ ] Show "Scraping..." indicator during operation
- [ ] Block TUI during scraping (foreground operation)
- [ ] Refresh data after scrape completes
- [ ] Handle scrape errors gracefully
- [ ] Test scraping from TUI works

### Task 4.4: Detail Views (Drill-down)
- [ ] Add detail view state to App
- [ ] Implement Enter key handling for drill-down:
  - [ ] Org → List of repos in that org
  - [ ] Repo → List of contributors for that repo
  - [ ] Contributor → List of repo contributions
- [ ] Add back navigation (Escape key)
- [ ] Update UI for detail views
- [ ] Test drill-down navigation works

### Task 4.5: Error Handling and Polish
- [ ] Add error display in UI
- [ ] Handle database connection errors
- [ ] Handle empty data gracefully
- [ ] Add loading indicators where needed
- [ ] Improve table formatting and alignment
- [ ] Test error scenarios

## Phase 5: Final Testing and Documentation

### Task 5.1: Integration Testing
- [ ] Test complete workflow: scrape → browse → drill-down
- [ ] Test all keyboard shortcuts work correctly
- [ ] Test with different data scenarios (empty, large datasets)
- [ ] Test error recovery

### Task 5.2: Documentation
- [ ] Update README with TUI instructions
- [ ] Document keyboard shortcuts
- [ ] Add usage examples
- [ ] Update help text in TUI

### Task 5.3: Code Cleanup
- [ ] Remove unused code from migration
- [ ] Add proper error handling throughout
- [ ] Add code documentation
- [ ] Run clippy and fix warnings

## Implementation Notes

- Each task should be completable independently and result in working code
- Test each task before moving to the next
- Focus on getting basic functionality working before adding polish
- Commit after completing each major task group
- Some tasks may need to be broken down further during implementation

## Current Status
- [x] Phase 1: Basic TUI Structure
- [ ] Phase 2: Database Integration  
- [ ] Phase 3: Core TUI Functionality
- [ ] Phase 4: Sorting and Polish
- [ ] Phase 5: Final Testing and Documentation

*Start with Phase 1, Task 1.1*